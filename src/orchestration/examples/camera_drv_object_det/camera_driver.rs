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

use logging_tracing::prelude::*;
use orchestration::actions::invoke::InvokeResult;
pub struct CameraDriver {}

impl CameraDriver {
    pub fn new() -> Self {
        Self {}
    }

    pub fn read_input(&mut self) -> InvokeResult {
        info!("read_input start");
        Ok(())
    }

    pub fn process(&mut self) -> InvokeResult {
        info!("process start");
        Ok(())
    }

    pub fn write_output(&mut self) -> InvokeResult {
        info!("write_output start");
        Ok(())
    }
}
