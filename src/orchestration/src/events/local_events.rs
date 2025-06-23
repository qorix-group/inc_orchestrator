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

#![allow(dead_code)]
use std::{future::Future, sync::Arc};

use async_runtime::channels::spmc_broadcast::*;

use crate::events::event_traits::{ListenerTrait, NotifierTrait};
use crate::{
    actions::internal::action::{ActionExecError, ActionResult},
    core::orch_locks::OrchTryLock,
};
use foundation::prelude::*;

const MAX_NUM_OF_EVENTS: usize = 8;

pub struct LocalEvent {
    sender: Option<Sender<u32, MAX_NUM_OF_EVENTS>>,
    receiver: Receiver<u32, MAX_NUM_OF_EVENTS>,
}

impl Default for LocalEvent {
    fn default() -> Self {
        Self::new()
    }
}

impl LocalEvent {
    pub fn new() -> Self {
        let (s, r) = create_channel::<u32, MAX_NUM_OF_EVENTS>(8);
        Self {
            sender: Some(s),
            receiver: r,
        }
    }

    pub fn get_notifier(&mut self) -> Option<LocalNotifier> {
        self.sender.take().map(|v| LocalNotifier { sender: Arc::new(v) })
    }

    pub fn get_listener(&mut self) -> Option<LocalListener> {
        self.receiver.try_clone().map(|v| LocalListener {
            receiver: Arc::new(OrchTryLock::new(v)),
        })
    }
}

pub struct LocalNotifier {
    sender: Arc<Sender<u32, MAX_NUM_OF_EVENTS>>,
}

impl LocalNotifier {
    async fn execute_impl(notifier: Arc<Sender<u32, MAX_NUM_OF_EVENTS>>, v: u32) -> ActionResult {
        debug!("LocalSync: Notifier sending value: {}", v);
        notifier.send(&v).map_err(|_| ActionExecError::NonRecoverableFailure)
    }
}

impl NotifierTrait for LocalNotifier {
    #[allow(clippy::manual_async_fn)]
    fn notify(&self, value: u32) -> impl Future<Output = ActionResult> + Send + 'static {
        Self::execute_impl(self.sender.clone(), value)
    }
}

pub struct LocalListener {
    receiver: Arc<OrchTryLock<Receiver<u32, MAX_NUM_OF_EVENTS>>>, // Arc used here to "share between futures, not between actions"
}

impl LocalListener {
    async fn execute_impl(listener: Arc<OrchTryLock<Receiver<u32, MAX_NUM_OF_EVENTS>>>) -> ActionResult {
        match listener.try_lock() {
            Ok(mut receiver) => {
                if (receiver.recv().await).is_some() {
                    debug!("LocalSync: Listener received an event");
                    Ok(())
                } else {
                    error!("LocalSync: Listener lost its notifier!");
                    Err(ActionExecError::NonRecoverableFailure)
                }
            }
            Err(_) => {
                error!("LocalSync: Listener is already locked, fatal failure!");
                Err(ActionExecError::NonRecoverableFailure)
            }
        }
    }
}

impl ListenerTrait for LocalListener {
    fn next(&mut self) -> impl Future<Output = ActionResult> + Send + 'static {
        let c = self.receiver.clone();

        Self::execute_impl(c)
    }
}

#[cfg(test)]
#[cfg(not(loom))]
mod tests {
    use crate::testing::OrchTestingPoller;

    use super::*;

    #[test]
    fn local_listener_success() {
        let mut event = LocalEvent::new();
        let notifier = event.get_notifier().expect("Notifier should be available");
        let mut listener = event.get_listener().expect("Listener should be available");

        // Notify with a value and then listen for it
        assert!(OrchTestingPoller::block_on(async move {
            let notify_result = notifier.notify(42).await;
            assert!(notify_result.is_ok(), "Notify should succeed");

            let listen_result = listener.next().await;
            assert!(listen_result.is_ok(), "Listener should receive event");
        })
        .is_some());
    }

    #[test]
    fn local_listener_lost_notifier() {
        let mut event = LocalEvent::new();
        let _notifier = event.get_notifier().expect("Notifier should be available");
        let mut listener = event.get_listener().expect("Listener should be available");

        // Drop the notifier before listening
        drop(_notifier);

        assert!(OrchTestingPoller::block_on(async move {
            let listen_result = listener.next().await;
            assert!(listen_result.is_err(), "Listener should fail if notifier is gone");
        })
        .is_some());
    }

    #[test]
    fn local_listener_already_locked() {
        let mut event = LocalEvent::new();
        let _notifier = event.get_notifier().expect("Notifier should be available");
        let mut listener = event.get_listener().expect("Listener should be available");

        // Simulate lock contention by locking manually
        let receiver_arc = listener.receiver.clone();
        let _lock = receiver_arc.try_lock().expect("Should be able to lock");

        assert!(OrchTestingPoller::block_on(async move {
            let listen_result = listener.next().await;
            assert!(listen_result.is_err(), "Listener should fail if already locked");
        })
        .is_some());
    }
}
