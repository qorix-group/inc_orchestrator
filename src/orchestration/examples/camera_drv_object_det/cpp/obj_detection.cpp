/********************************************************************************
* Copyright (c) 2025 Contributors to the Eclipse Foundation
*
* See the NOTICE file(s) distributed with this work for additional
* information regarding copyright ownership.
*
* This program and the accompanying materials are made available under the
* terms of the Apache License Version 2.0 which is available at
* https://www.apache.org/licenses/LICENSE-2.0
*
* SPDX-License-Identifier: Apache-2.0
********************************************************************************/

#include "include/obj_detection.h"

ObjDetectionCC::ObjDetectionCC() : state(ObjDetectionState::INITIAL) {}
void ObjDetectionCC::pre_processing_cc()
{
    state = ObjDetectionState::PRE_PROCESSING;
    // Add pre-processing logic here
}
void ObjDetectionCC::drive_q1_cc()
{
    state = ObjDetectionState::DRIVE_Q1;
    // Add logic for DRIVE_Q1 here
}
void ObjDetectionCC::drive_q2_cc()
{
    state = ObjDetectionState::DRIVE_Q2;
    // Add logic for DRIVE_Q2 here
}
void ObjDetectionCC::drive_q3_cc()
{
    state = ObjDetectionState::DRIVE_Q3;
    // Add logic for DRIVE_Q3 here
}
void ObjDetectionCC::object_fusion_cc()
{
    state = ObjDetectionState::OBJECT_FUSION;
    // Add object fusion logic here
}

// C interface for Rust FFI
extern "C"
{
    void *create_obj_detection()
    {
        return static_cast<void *>(new ObjDetectionCC());
    }

    void obj_detection_pre_processing(void *obj_detection_ptr)
    {
        static_cast<ObjDetectionCC *>(obj_detection_ptr)->pre_processing_cc();
    }

    void obj_detection_drive_q1(void *obj_detection_ptr)
    {
        static_cast<ObjDetectionCC *>(obj_detection_ptr)->drive_q1_cc();
    }

    void obj_detection_drive_q2(void *obj_detection_ptr)
    {
        static_cast<ObjDetectionCC *>(obj_detection_ptr)->drive_q2_cc();
    }

    void obj_detection_drive_q3(void *obj_detection_ptr)
    {
        static_cast<ObjDetectionCC *>(obj_detection_ptr)->drive_q3_cc();
    }

    void obj_detection_object_fusion(void *obj_detection_ptr)
    {
        static_cast<ObjDetectionCC *>(obj_detection_ptr)->object_fusion_cc();
    }

    void free_obj_detection(void *obj_detection_ptr)
    {
        delete static_cast<ObjDetectionCC *>(obj_detection_ptr);
    }
}
