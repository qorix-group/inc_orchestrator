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

use ::core::{
    ops::{Add, Sub},
    time::Duration,
};

/// A measurement of a monotonically nondecreasing clock. This is thin wrapper for std version so our runtime can choose
/// to use different source without changing the code base.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Instant(std::time::Instant);

impl Instant {
    /// Returns the duration since the clock started.
    pub fn elapsed(&self) -> ::core::time::Duration {
        self.0.elapsed()
    }

    pub fn saturating_duration_since(&self, earlier: Instant) -> Duration {
        self.0.saturating_duration_since(earlier.0)
    }

    pub fn duration_since(&self, earlier: Instant) -> Duration {
        self.0.duration_since(earlier.0)
    }

    pub fn checked_duration_since(&self, earlier: Instant) -> Option<Duration> {
        self.0.checked_duration_since(earlier.0)
    }

    pub fn checked_add(&self, duration: Duration) -> Option<Instant> {
        self.0.checked_add(duration).map(Instant)
    }
}

impl Sub<Duration> for Instant {
    type Output = Instant;

    fn sub(self, rhs: Duration) -> Self::Output {
        Instant(self.0.checked_sub(rhs).expect("overflow when subtracting duration from instant"))
    }
}

impl Add<Duration> for Instant {
    type Output = Instant;

    fn add(self, rhs: Duration) -> Self::Output {
        Instant(self.0.checked_add(rhs).expect("overflow when adding duration to instant"))
    }
}

impl Sub for Instant {
    type Output = Duration;

    fn sub(self, rhs: Instant) -> Self::Output {
        self.0.duration_since(rhs.0)
    }
}

/// A clock that provides the current time as an `Instant`.
pub struct Clock;

#[cfg(not(feature = "vm_time_correction"))]
impl Clock {
    /// Returns the current time as an `Instant`.
    pub fn now() -> Instant {
        Instant(std::time::Instant::now())
    }

    /// Returns the current time as a `Duration` since the clock started.
    pub fn now_duration() -> Duration {
        Self::now().elapsed()
    }
}

#[cfg(feature = "vm_time_correction")]
static LAST_INSTANT: std::sync::Mutex<Option<std::time::Instant>> = std::sync::Mutex::new(None);
#[cfg(feature = "vm_time_correction")]
static INIT: std::sync::Once = std::sync::Once::new();

#[cfg(feature = "vm_time_correction")]
impl Clock {
    fn last_instant() -> &'static std::sync::Mutex<Option<std::time::Instant>> {
        INIT.call_once(|| {
            LAST_INSTANT.lock().unwrap().replace(std::time::Instant::now());
        });

        &LAST_INSTANT
    }

    /// Returns the current time as an `Instant`.
    /// Ensures time never goes backward.
    pub fn now() -> Instant {
        let mutex = Self::last_instant();

        let now = std::time::Instant::now();
        let mut last = mutex.lock().unwrap();

        let mut last_val = last.unwrap();

        if now > last_val {
            last.replace(now);
            last_val = now;
        }

        drop(last);

        Instant(last_val)
    }

    /// Returns the duration since the last stored instant.
    pub fn now_duration() -> Duration {
        Self::now().elapsed()
    }
}
