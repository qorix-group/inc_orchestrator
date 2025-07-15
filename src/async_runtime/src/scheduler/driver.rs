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

use ::core::{ops::Deref, task::Waker};
use std::sync::Arc;

use foundation::prelude::FoundationAtomicU64;
use foundation::prelude::ScopeGuardBuilder;
use foundation::prelude::*;

use crate::{
    scheduler::{
        context::{ctx_get_handler, ctx_get_wakeup_time, ctx_get_worker_id, ctx_set_wakeup_time, ctx_unset_wakeup_time},
        scheduler_mt::AsyncScheduler,
        workers::worker_types::{WorkerInteractor, *},
    },
    time::{clock::Instant, TimeDriver},
};

#[derive(Clone)]
pub(crate) struct Drivers {
    inner: Arc<Inner>,
}

impl Drivers {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Inner::new()),
        }
    }
}

impl Deref for Drivers {
    type Target = Inner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub(crate) struct Inner {
    time: TimeDriver,

    next_promised_wakeup: FoundationAtomicU64, // next time we promise to wakeup "some" worker
}

//TODO: This has to be reworked once we have IoDriver since those two are tightly coupled between each other
impl Inner {
    pub fn new() -> Self {
        Self {
            time: TimeDriver::new(4096),
            next_promised_wakeup: FoundationAtomicU64::new(0),
        }
    }

    pub fn register_timeout(&self, expire_at: Instant, waker: Waker) -> Result<(), CommonErrors> {
        self.time.register_timeout(expire_at, waker)?;

        if ctx_get_worker_id().typ() == WorkerType::Dedicated {
            // Unparking needs to happen as all async workers may be parked so no one ever will look at this timeout
            ctx_get_handler().unwrap().unpark_some_async_worker();
        }

        Ok(())
    }

    pub fn process_work(&self) {
        self.time.process_timeouts();
    }

    pub fn park(&self, scheduler: &AsyncScheduler, worker: &WorkerInteractor) {
        let _exit_guard = ScopeGuardBuilder::new(true)
            .on_init(|_| Ok(()) as Result<(), ()>)
            .on_drop(|_| {
                self.time.process_timeouts();
            })
            .create()
            .unwrap();

        let expire_time = self.time.next_process_time();

        if expire_time.is_none() {
            // No expiration is there, I sleep indefinitely
            debug!("No next expiration time, parking indefinitely");
            self.park_on_cv(scheduler, worker);
            return;
        }

        let expire_time_instant = expire_time.unwrap();
        let expire_time_u64 = self.time.instant_into_u64(&expire_time_instant);

        // This is th last time we promised to wakeup this worker
        let previous_wakeup_time = self.get_last_wakeup_time_for_worker();
        let global_promis_next_time_wakeup = self.next_promised_wakeup.load(::core::sync::atomic::Ordering::Relaxed);

        if (expire_time_u64 == global_promis_next_time_wakeup) && (previous_wakeup_time != expire_time_u64) {
            debug!("Someone else waiting on timewheel, we will park on cv without timeout");
            self.park_on_cv(scheduler, worker);
            return;
        }

        self.park_on_cv_timeout(scheduler, worker, &expire_time_instant, expire_time_u64);
    }

    fn park_on_cv(&self, scheduler: &AsyncScheduler, worker: &WorkerInteractor) {
        let worker_id = ctx_get_worker_id().worker_id() as usize;

        let mut _guard = worker.mtx.lock().unwrap();

        if !self.park_on_cv_pre_stage(scheduler, worker, worker_id) {
            // We were notified before we could park, so we just return
            return;
        }

        _guard = worker
            .cv
            .wait_while(_guard, |_| self.park_on_cv_condition_check(scheduler, worker, worker_id))
            .unwrap();
    }

    fn park_on_cv_timeout(&self, scheduler: &AsyncScheduler, worker: &WorkerInteractor, expire_time_instant: &Instant, expire_time_u64: u64) {
        let worker_id = ctx_get_worker_id().worker_id() as usize;

        // We do it before CME to keep state consistent
        let dur = self.time.duration_since_now(expire_time_instant);
        if dur.is_zero() {
            warn!("Tried to park on cv with duration zero or lower, looks like a worker was stuck for some time, unparking immediately");
            return;
        }

        let mut _guard = worker.mtx.lock().unwrap();

        if !self.park_on_cv_pre_stage(scheduler, worker, worker_id) {
            // We were notified before we could park, so we just return
            return;
        }

        // We sleep to new timeout
        self.next_promised_wakeup.store(expire_time_u64, ::core::sync::atomic::Ordering::Relaxed);
        self.stash_promised_wakeup_time_for_worker(expire_time_u64);

        let wait_result;

        debug!("Definite sleep decision, try sleep {} ms", dur.as_millis());

        // TODO: To improve accuracy of sleep, we shall switch to select and provide correct unpark condition. This could be done once we work on IoDriver since
        // some code can be shared between IoDriver and TimeDriver.
        (_guard, wait_result) = worker
            .cv
            .wait_timeout_while(_guard, dur, |_| self.park_on_cv_condition_check(scheduler, worker, worker_id))
            .unwrap();

        if wait_result.timed_out() {
            // We did timeout due to sleep request, so we fullfilled driver promise
            self.clear_last_wakeup_time_for_worker();
            worker.state.0.store(WORKER_STATE_EXECUTING, ::core::sync::atomic::Ordering::SeqCst);
            scheduler.transition_from_parked(worker_id);
            debug!("Woken up from sleep after timeout");
        }
    }

    /// # Safety
    /// This function must be called under mutex lock of the worker.
    fn park_on_cv_pre_stage(&self, scheduler: &AsyncScheduler, worker: &WorkerInteractor, worker_id: usize) -> bool {
        match worker.state.0.compare_exchange(
            WORKER_STATE_EXECUTING,
            WORKER_STATE_SLEEPING_CV,
            ::core::sync::atomic::Ordering::SeqCst,
            ::core::sync::atomic::Ordering::SeqCst,
        ) {
            Ok(_) => true,
            Err(WORKER_STATE_NOTIFIED) => {
                // We were notified before, so we shall continue
                scheduler.transition_from_parked(worker_id);

                worker.state.0.store(WORKER_STATE_EXECUTING, ::core::sync::atomic::Ordering::SeqCst);
                debug!("Notified while try to sleep, searching again");
                false
            }
            Err(WORKER_STATE_SHUTTINGDOWN) => {
                // If we should shutdown, we simply need to return. And the run loop exits itself.
                false
            }
            Err(s) => {
                panic!("Inconsistent state when parking: {}", s);
            }
        }
    }

    /// # Safety
    /// This function must be called under mutex lock of the worker.
    fn park_on_cv_condition_check(&self, scheduler: &AsyncScheduler, worker: &WorkerInteractor, worker_id: usize) -> bool {
        match worker.state.0.compare_exchange(
            WORKER_STATE_NOTIFIED,
            WORKER_STATE_EXECUTING,
            ::core::sync::atomic::Ordering::SeqCst,
            ::core::sync::atomic::Ordering::SeqCst,
        ) {
            Ok(_) => {
                scheduler.transition_from_parked(worker_id);
                debug!("Woken up from sleep by notification in park_on_cv_timeout");
                false
            }
            Err(WORKER_STATE_SHUTTINGDOWN) => {
                // break here and run loop will exit
                false
            }
            Err(_) => {
                true // spurious wake-up
            }
        }
    }

    fn stash_promised_wakeup_time_for_worker(&self, at: u64) {
        ctx_set_wakeup_time(at);
    }

    fn get_last_wakeup_time_for_worker(&self) -> u64 {
        ctx_get_wakeup_time()
    }

    fn clear_last_wakeup_time_for_worker(&self) {
        ctx_unset_wakeup_time();
    }
}
