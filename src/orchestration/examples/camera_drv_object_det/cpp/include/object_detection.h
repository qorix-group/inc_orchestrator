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
#pragma once

enum class ObjectDetectionState
{
    INITIAL,
    PRE_PROCESSING,
    DRIVE_Q1,
    DRIVE_Q2,
    DRIVE_Q3,
    OBJECT_FUSION,
};

class ObjectDetection
{
    ObjectDetectionState state;

public:
    ObjectDetection();
    void pre_processing();
    void drive_q1();
    void drive_q2();
    void drive_q3();
    void object_fusion();
};
