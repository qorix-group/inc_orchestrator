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

use crate::mio::types::{IoEventInterest, IoId, IoRegistryEntry, IoSelector, Result};

#[derive(Clone)]
pub struct Registry<T: IoSelector + ?Sized> {
    pub(crate) selector: T,
}

impl<T: IoSelector> Registry<T> {
    pub fn new(selector: T) -> Self {
        Registry { selector }
    }

    pub fn register<Source: IoRegistryEntry<T>>(&self, source: &mut Source, id: IoId, interest: IoEventInterest) -> Result<()> {
        source.register(self, id, interest)
    }

    pub fn reregister<Source: IoRegistryEntry<T>>(&self, source: &mut Source, id: IoId, interest: IoEventInterest) -> Result<()> {
        source.reregister(id, interest)
    }

    pub fn deregister<Source: IoRegistryEntry<T>>(&self, source: &mut Source) -> Result<()> {
        source.deregister()
    }
}
