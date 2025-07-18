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

use std::ffi::c_void;

#[link(name = "libobj_detection_cc")]
extern "C" {
    pub fn create_obj_detection() -> *mut c_void;

    pub fn obj_detection_pre_processing(obj_detection_ptr: *mut c_void);

    pub fn obj_detection_drive_q1(obj_detection_ptr: *mut c_void);

    pub fn obj_detection_drive_q2(obj_detection_ptr: *mut c_void);

    pub fn obj_detection_drive_q3(obj_detection_ptr: *mut c_void);

    pub fn obj_detection_object_fusion(obj_detection_ptr: *mut c_void);

    pub fn free_obj_detection(obj_detection_ptr: *mut c_void);
}
