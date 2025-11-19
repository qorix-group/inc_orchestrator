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

use core::time::Duration;
use std::sync::{atomic::AtomicU64, Arc};

use async_runtime::{
    futures::sleep,
    time::clock::{Clock, Instant},
};
use foundation::prelude::*;

use crate::events::event_traits::ListenerTrait;

pub(crate) struct TimerEvent {
    start_time: Option<Instant>,
    cycle_duration: core::time::Duration,
    tick: Arc<AtomicU64>,
}

impl TimerEvent {
    pub fn new(cycle_duration: core::time::Duration) -> Self {
        TimerEvent {
            start_time: None,
            cycle_duration,
            tick: Arc::new(AtomicU64::new(0)),
        }
    }
}

impl ListenerTrait for TimerEvent {
    fn next(&mut self) -> impl core::future::Future<Output = crate::prelude::ActionResult> + Send + 'static {
        // TODO: Fix the issue that next() i called before iteration to create future

        let is_first_time = self.start_time.is_none();
        if is_first_time {
            let now: Instant = Clock::now();
            let now_ns = now.elapsed().as_nanos();
            let rest = now_ns % self.cycle_duration.as_nanos();
            let missing = now_ns - rest;
            self.start_time = Some(now.checked_add(Duration::from_nanos(missing as u64)).unwrap());
            info!("Starting TimerEvent at {:?}, first tick in {} ns", self.start_time.unwrap(), missing);
        } else {
            self.tick.fetch_add(1, std::sync::atomic::Ordering::AcqRel);
        }

        let start_time = self.start_time.unwrap();
        let cycle = self.cycle_duration;
        let tick = self.tick.clone();

        // TODO: fix when mio is providing timer events/ or timewheel provides interval, currently we use sleep
        async move {
            let now = Clock::now();
            let elapsed = now.saturating_duration_since(start_time).as_millis();
            let elapsed_ticks = (elapsed / cycle.as_millis()) as u64;
            let expected_ticks = tick.load(std::sync::atomic::Ordering::Relaxed);

            info!(
                "TimerEvent Tick: now={:?} elapsed={} elapsed_ticks={}, expected_ticks={}",
                now, elapsed, elapsed_ticks, expected_ticks
            );

            if elapsed_ticks > expected_ticks {
                tick.store(elapsed_ticks, std::sync::atomic::Ordering::Release);
                warn!(
                    "Expected that we are at timer event tick {} but we are on {}. Missed {} ticks!",
                    expected_ticks,
                    elapsed_ticks,
                    elapsed_ticks - expected_ticks
                );
            } else {
                let elapsed_in_full_cycles = cycle.as_millis() * (expected_ticks) as u128;

                match elapsed.cmp(&elapsed_in_full_cycles) {
                    core::cmp::Ordering::Less => {
                        let remaining = elapsed_in_full_cycles - elapsed;
                        let remaining_duration = Duration::from_millis(remaining as u64);

                        info!("Will sleep for {:?}", remaining_duration);
                        sleep::sleep(remaining_duration).await;
                    }
                    core::cmp::Ordering::Equal => {}
                    core::cmp::Ordering::Greater => {}
                };
            }

            Ok(())
        }
    }
}
