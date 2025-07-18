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

use crate::ffi::{
    create_obj_detection, free_obj_detection, obj_detection_drive_q1, obj_detection_drive_q2, obj_detection_drive_q3, obj_detection_object_fusion,
    obj_detection_pre_processing,
};
use logging_tracing::prelude::*;
use orchestration::actions::invoke::InvokeResult;
use std::ffi::c_void;

unsafe impl Send for ObjectDetection {}
pub struct ObjectDetection {
    obj_detection_ptr: *mut c_void,
}

impl ObjectDetection {
    pub fn new() -> Self {
        Self {
            obj_detection_ptr: unsafe { create_obj_detection() },
        }
    }

    pub fn pre_processing(&mut self) -> InvokeResult {
        info!("PreProcessing start");
        unsafe {
            obj_detection_pre_processing(self.obj_detection_ptr);
        }
        info!("PreProcessing end");
        Ok(())
    }
    pub fn drive_q1(&mut self) -> InvokeResult {
        info!("DriveQ1 start");
        unsafe {
            obj_detection_drive_q1(self.obj_detection_ptr);
        }
        info!("DriveQ1 end");
        Ok(())
    }
    pub fn drive_q2(&mut self) -> InvokeResult {
        info!("DriveQ2 start");
        unsafe {
            obj_detection_drive_q2(self.obj_detection_ptr);
        }
        info!("DriveQ2 end");
        Ok(())
    }
    pub fn drive_q3(&mut self) -> InvokeResult {
        info!("DriveQ3 start");
        unsafe {
            obj_detection_drive_q3(self.obj_detection_ptr);
        }
        info!("DriveQ3 end");
        Ok(())
    }
    pub fn object_fusion(&mut self) -> InvokeResult {
        info!("ObjectFusion start");
        unsafe {
            obj_detection_object_fusion(self.obj_detection_ptr);
        }
        info!("ObjectFusion end");
        Ok(())
    }
}

impl Drop for ObjectDetection {
    fn drop(&mut self) {
        unsafe {
            free_obj_detection(self.obj_detection_ptr);
        }
    }
}
