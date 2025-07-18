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

fn main() {
    println!("cargo::rerun-if-changed=cpp/include/obj_detection.h");
    println!("cargo::rerun-if-changed=cpp/obj_detection.cpp");
    println!("cargo::rerun-if-changed=build.rs");

    cc::Build::new().cpp(true).file("cpp/obj_detection.cpp").compile("libobj_detection_cc");
}
