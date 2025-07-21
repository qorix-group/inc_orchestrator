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

use crate::mio::{
    registry::Registry,
    types::{IoId, IoSelector, IoSelectorEventContainer, Result},
};
use core::time::Duration;

/// An object that allows for registering I/O resources and polling them for events with chosen selector.
pub struct Poll<T: IoSelector> {
    registry: Registry<T>,
}

impl<T: IoSelector> Poll<T> {
    /// Create a poll instance for a selector.
    pub fn new(selector: T) -> Self {
        Poll {
            registry: Registry::new(selector),
        }
    }

    /// Wait on events on the registered I/O resources using the chosen selector.
    pub fn poll<Container: IoSelectorEventContainer>(&self, events: &mut Container, timeout: Option<Duration>) -> Result<()> {
        events.clear();
        self.registry.selector.select(events, timeout)
    }

    /// Get the capacity that the events container passed to poll should have to be able to contain all events.
    pub fn get_safe_poll_events_capacity(&self) -> usize {
        self.registry.selector.capacity()
    }

    /// Create a waker that can unblock the poll call.
    pub fn create_waker(&self, id: IoId) -> Result<T::Waker> {
        self.registry.selector.create_waker(id)
    }

    /// Returns the registry that can be used to manage the polled I/O resources.
    pub fn registry(&self) -> &Registry<T> {
        &self.registry
    }
}
