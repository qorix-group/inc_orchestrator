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

#include <iostream>
#include "include/object_detection.h"

extern "C" void rust_log_info(const char *msg);

ObjectDetection::ObjectDetection() : state(ObjectDetectionState::INITIAL) {}
void ObjectDetection::pre_processing()
{
    state = ObjectDetectionState::PRE_PROCESSING;
    // Add pre-processing logic here
    rust_log_info("Pre-processing step completed.");
}
void ObjectDetection::drive_q1()
{
    state = ObjectDetectionState::DRIVE_Q1;
    // Add logic for DRIVE_Q1 here
    rust_log_info("Driving Q1 step completed.");
}
void ObjectDetection::drive_q2()
{
    state = ObjectDetectionState::DRIVE_Q2;
    // Add logic for DRIVE_Q2 here
    rust_log_info("Driving Q2 step completed.");
}
void ObjectDetection::drive_q3()
{
    state = ObjectDetectionState::DRIVE_Q3;
    // Add logic for DRIVE_Q3 here
    rust_log_info("Driving Q3 step completed.");
}
void ObjectDetection::object_fusion()
{
    state = ObjectDetectionState::OBJECT_FUSION;
    // Add object fusion logic here
    rust_log_info("Object fusion step completed.");
}

// Expose the methods of ObjectDetection to orchestration
#include "expose_object_to_orchestration.h"
EXPOSE_OBJECT_TO_ORCHESTRATION(ObjectDetection, pre_processing, drive_q1, drive_q2, drive_q3, object_fusion)
