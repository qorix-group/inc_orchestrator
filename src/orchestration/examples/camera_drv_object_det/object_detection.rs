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
pub struct ObjectDetection {}

impl ObjectDetection {
    pub fn new() -> Self {
        Self {}
    }

    pub fn pre_processing(&mut self) -> InvokeResult {
        info!("PreProcessing start");
        Ok(())
    }
    pub fn drive_q1(&mut self) -> InvokeResult {
        info!("DriveQ1 start");
        Ok(())
    }
    pub fn drive_q2(&mut self) -> InvokeResult {
        info!("DriveQ2 start");
        Ok(())
    }
    pub fn drive_q3(&mut self) -> InvokeResult {
        info!("DriveQ3 start");
        Ok(())
    }
    pub fn object_fusion(&mut self) -> InvokeResult {
        info!("ObjectFusion start");
        Ok(())
    }
}
