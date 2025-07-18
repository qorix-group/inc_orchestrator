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

enum class ObjDetectionState
{
    INITIAL,
    PRE_PROCESSING,
    DRIVE_Q1,
    DRIVE_Q2,
    DRIVE_Q3,
    OBJECT_FUSION,
};

class ObjDetectionCC
{
    ObjDetectionState state;

public:
    ObjDetectionCC();
    void pre_processing_cc();
    void drive_q1_cc();
    void drive_q2_cc();
    void drive_q3_cc();
    void object_fusion_cc();
};
