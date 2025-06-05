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

pub mod orch_tag;
pub mod tag;

#[derive(Clone, Debug, Copy, PartialEq)]
pub struct ProgramDatabaseParams {
    pub clonable_invokes_capacity: usize,
}

impl Default for ProgramDatabaseParams {
    fn default() -> Self {
        Self {
            clonable_invokes_capacity: 256,
        }
    }
}

#[derive(Clone, Debug, Copy, PartialEq)]
pub struct DesignConfig {
    pub db_params: ProgramDatabaseParams,
    /// Describes how many times the task chain can be repeated after a failure when user explicitly requests it from [`Catch`] action
    pub max_failure_retry: u32,
}

impl Default for DesignConfig {
    fn default() -> Self {
        const DEFAULT_MAX_FAILURE_RETRY: u32 = 3;
        DesignConfig {
            db_params: ProgramDatabaseParams::default(),
            max_failure_retry: DEFAULT_MAX_FAILURE_RETRY,
        }
    }
}
