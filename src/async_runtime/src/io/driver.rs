//
// Copyright (c) 2025 Contributors to the Eclipse Foundation
//
// See the NOTICE file(s) distributed with this work for additional
// information regarding copyright ownership.
//
// This program and the accompanying materials are made available under the
// terms of the Apache License Version 2.0 which is available at
// <https://www.apache.org/licenses/LICENSE-2.0>
//
// SPDX-License-Identifier: Apache-2.0
//

// TODO: To be removed once used in IO APIs
#![allow(dead_code)]

use core::time::Duration;
use std::sync::{Arc, Mutex};

use core::sync::atomic::Ordering;

use crate::{
    io::{
        async_registration::{ReadinessState, RegistrationInfo},
        AsyncSelector,
    },
    mio::{
        poll::Poll,
        registry::Registry,
        types::{IoEvent, IoEventInterest, IoId, IoRegistryEntry, IoSelector},
    },
};
use foundation::prelude::*;

use iceoryx2_bb_container::slotmap::{SlotMap, SlotMapKey};

// Holds MIO object and all parts needed to manage it
pub struct IoDriver {
    inner: Mutex<IoDriverInner>,
    async_registration: Arc<Registrations>,
    registry: Registry<AsyncSelector>,
    waker: Arc<<AsyncSelector as IoSelector>::Waker>,
}

pub(crate) struct IoDriverInner {
    pool: Poll<AsyncSelector>,
    events: Vec<IoEvent>,
}

/// Proxy handle that let all async code interact with underlying MIO selector in a safe way.
#[derive(Clone)]
pub struct IoDriverHandle<T: IoSelector = AsyncSelector> {
    registry: Registry<T>,
    async_registration: Arc<Registrations>,
}

// Proxy that is able to unpark connected IO Driver
pub struct IoDriverUnparker {
    waker: Arc<<AsyncSelector as IoSelector>::Waker>,
}

struct RegistrationData {
    tracking: SlotMap<Arc<RegistrationInfo>>,
    waiting_release: Vec<SlotMapKey>, // Could be lock free Container from iceoryx to not need mutex on removal.
}

struct Registrations {
    pending_release_count: FoundationAtomicUsize,
    data: Mutex<RegistrationData>,
}

impl Registrations {
    pub fn new() -> Self {
        Registrations {
            pending_release_count: FoundationAtomicUsize::new(0),
            data: Mutex::new(RegistrationData {
                tracking: SlotMap::new(1234),
                waiting_release: Vec::new(1234),
            }),
        }
    }

    fn request_registration_info(&self) -> Result<Arc<RegistrationInfo>, CommonErrors> {
        let mut data = self.data.lock().unwrap();

        let item = Arc::new(RegistrationInfo::default());

        let key = data.tracking.insert(item.clone()).ok_or(CommonErrors::NoSpaceLeft)?;

        item.insert_tracking_key(key.value());

        Ok(item)
    }

    fn schedule_registration_for_disposal(&self, key: SlotMapKey) {
        let mut data = self.data.lock().unwrap();
        data.waiting_release.push(key);
        self.pending_release_count.fetch_add(1, Ordering::SeqCst);
    }

    fn cleanup_disposed_registrations(&self) {
        if self.pending_release_count.load(Ordering::SeqCst) == 0 {
            return;
        }

        let mut data = self.data.lock().unwrap();

        while let Some(key) = data.waiting_release.pop() {
            data.tracking.remove(key);
        }

        //Consider ordering change
        self.pending_release_count.store(0, Ordering::SeqCst);
    }

    // # Caveats
    // Expensive call that takes a lock, should only be used for error condition cleanups
    fn release_registration_info(&self, key: SlotMapKey) {
        let mut data = self.data.lock().unwrap();
        data.tracking.remove(key);

        //TODO: shall we check pending for double sure ?
    }
}

impl<T: IoSelector> IoDriverHandle<T> {
    pub(super) fn new(registry: Registry<T>) -> Self {
        IoDriverHandle {
            registry,
            async_registration: Arc::new(Registrations::new()),
        }
    }

    pub(crate) fn add_io_source<Source>(&self, source: &mut Source, interest: IoEventInterest) -> Result<Arc<RegistrationInfo>, CommonErrors>
    where
        Source: IoRegistryEntry<T> + core::fmt::Debug,
    {
        self.async_registration
            .request_registration_info()
            .and_then(|info| match self.registry.register(source, info.identifier(), interest) {
                Ok(_) => {
                    info!(
                        "Successfully registered IO source ({:?}) with interest: {:?} and assigned info ident {:?}",
                        source,
                        interest,
                        info.identifier()
                    );

                    Ok(info)
                }
                Err(e) => {
                    //We need to remove this Arc from the tracking so it's not left dangling
                    let key = info.tracking_key();

                    if key.is_none() {
                        error!("Trying to release registration that was not registered ({:?})", source);
                        return Err(e);
                    }

                    self.async_registration.release_registration_info(SlotMapKey::new(key.unwrap()));
                    println!(
                        "Failed to register IO source ({:?}) with interest: {:?} and assigned info ident {:?}, error: {:?}",
                        source,
                        interest,
                        info.identifier(),
                        e
                    );
                    Err(e)
                }
            })
    }

    pub(crate) fn remove_io_source<Source>(&self, source: &mut Source, registration: &Arc<RegistrationInfo>)
    where
        Source: IoRegistryEntry<T> + core::fmt::Debug,
    {
        match self.registry.deregister(source) {
            Ok(_) => info!("Successfully deregistered IO source ({:?})", source),
            Err(e) => warn!("Deregister of source({:?}) failed with {:?}", source, e),
        }

        let key = registration.tracking_key();

        if key.is_none() {
            error!("Trying to remove registration that was not registered ({:?})", source);
            return;
        }

        // Right now we are sure that there is somewhere a next call to driver.poll, this source will not be considered.
        // Still it could be this source was considered int the poll happened during or little before deregister call so
        // we need to make sure that we keep it until next poll happens and remove before to release our reference count
        self.async_registration.schedule_registration_for_disposal(SlotMapKey::new(key.unwrap()));
    }
}

impl Default for IoDriver {
    fn default() -> Self {
        IoDriver::new(AsyncSelector::new(1234))
    }
}

type AccessGuard<'a> = std::sync::MutexGuard<'a, IoDriverInner>;

const WAKER_IO_ID: u64 = 0xAABBCCDD;

impl IoDriverUnparker {
    pub(crate) fn unpark(&self) {
        self.waker.wake();
    }
}

impl IoDriver {
    pub(crate) fn new(selector: AsyncSelector) -> Self {
        let poll = Poll::new(selector);
        let waker = poll.create_waker(IoId::new(WAKER_IO_ID)).unwrap(); //TODO: handle error;

        IoDriver {
            async_registration: Arc::new(Registrations::new()),
            registry: poll.registry().clone(),
            inner: Mutex::new(IoDriverInner {
                pool: poll,
                events: Vec::new(1000),
            }),
            waker: Arc::new(waker),
        }
    }

    pub(crate) fn get_unparker(&self) -> IoDriverUnparker {
        IoDriverUnparker { waker: self.waker.clone() }
    }

    pub fn handle(&self) -> IoDriverHandle<AsyncSelector> {
        IoDriverHandle {
            registry: self.registry.clone(),
            async_registration: self.async_registration.clone(),
        }
    }

    pub(crate) fn try_get_access(&self) -> Option<AccessGuard> {
        self.inner.try_lock().ok()
    }

    pub(crate) fn process_io(&self, inner: &mut AccessGuard, timeout: Option<Duration>) -> Result<(), CommonErrors> {
        // This shall ensure proper lifetime of unsafe section below
        self.async_registration.cleanup_disposed_registrations();

        let binding: &mut IoDriverInner = inner;

        match binding.pool.poll(&mut binding.events, timeout) {
            Err(CommonErrors::Timeout) => {
                return Err(CommonErrors::Timeout);
            }
            Ok(_) => {}
            _ => {
                panic!("Generic error not handled!!!");
            }
        }

        for event in binding.events.iter() {
            let id = event.id();

            if id == IoId::new(WAKER_IO_ID) {
                // This is our internal waker, just ignore it
                continue;
            }

            debug!("Processing mio identifier: {:?}", id);
            // SAFETY
            // Those two operations assumes that the lifetime of RegistrationInfo is held
            // by other part of code(Registrations in IoHandle), until this is fully removed
            // from the mio part and this code have done polling so we are sure we never end up here with invalid reference
            unsafe {
                let info = RegistrationInfo::from_identifier(id);
                let info_ref = &*info;

                let readiness = ReadinessState::from_mio_event(event);
                info_ref.wake(readiness);
            }
        }

        Ok(())
    }
}
