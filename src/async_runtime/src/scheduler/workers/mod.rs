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

pub mod dedicated_worker;
pub mod safety_worker;
pub mod worker;
pub mod worker_types;

use crate::scheduler::{workers::worker_types::WorkerId, SchedulerType};
use ::core::fmt::Debug;
use iceoryx2_bb_posix::thread::{Thread, ThreadBuilder, ThreadName, ThreadSpawnError};

#[derive(Default, Clone)]
pub struct ThreadParameters {
    pub(crate) priority: Option<u8>,
    pub(crate) scheduler_type: Option<SchedulerType>,
    pub(crate) affinity: Option<Box<[usize]>>,
    pub(crate) stack_size: Option<u64>,
}

impl ThreadParameters {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn priority(mut self, priority: u8) -> Self {
        self.priority = Some(priority);
        self
    }

    pub fn scheduler_type(mut self, scheduler_type: SchedulerType) -> Self {
        self.scheduler_type = Some(scheduler_type);
        self
    }

    /// An array of CPU core ids that the thread can run on.
    pub fn affinity(mut self, affinity: &[usize]) -> Self {
        self.affinity = Some(Box::from(affinity));
        self
    }

    pub fn stack_size(mut self, stack_size: u64) -> Self {
        self.stack_size = Some(stack_size);
        self
    }
}

pub(crate) fn spawn_thread<T, F>(tname: &'static str, id: &WorkerId, f: F, thread_params: &ThreadParameters) -> Result<Thread, ThreadSpawnError>
where
    T: Debug + Send + 'static,
    F: FnOnce() -> T + Send + 'static,
{
    let mut name = ThreadName::from_bytes(tname.as_bytes()).expect("thread name must be not longer than 15 chars");

    for digit in id.worker_id().to_string().into_bytes() {
        let _ = name.push(digit);
    }

    let mut tb = ThreadBuilder::new().name(&name);

    // If both scheduler type and priority are configured, disable inheritance from parent thread.
    if thread_params.scheduler_type.is_some() && thread_params.priority.is_some() {
        tb = tb.inherit_scheduling_attributes(false);
    }

    if let Some(priority) = thread_params.priority {
        tb = tb.priority(priority);
    }

    if let Some(scheduler_type) = &thread_params.scheduler_type {
        tb = tb.scheduler(scheduler_type.into());
    }

    if let Some(affinity) = &thread_params.affinity {
        tb = tb.affinity(affinity);
    }

    if let Some(stack_size) = thread_params.stack_size {
        return tb.stack_size(stack_size).spawn(f);
    }

    tb.spawn(f)
}
