# *******************************************************************************
# Copyright (c) 2025 Contributors to the Eclipse Foundation
#
# See the NOTICE file(s) distributed with this work for additional
# information regarding copyright ownership.
#
# This program and the accompanying materials are made available under the
# terms of the Apache License Version 2.0 which is available at
# https://www.apache.org/licenses/LICENSE-2.0
#
# SPDX-License-Identifier: Apache-2.0
# *******************************************************************************
load("@rules_cc//cc:cc_library.bzl", "cc_library")
load("@rules_rust//rust:defs.bzl", "rust_proc_macro")

rust_proc_macro(
    name = "orch_macros",
    srcs = glob(["src/**/*.rs"]),
    crate_name = "orchestration_macros",
    edition = "2021",
    visibility = ["//visibility:public"],
    deps = [
        "@score_crates//:quote",
        "@score_crates//:syn",
    ],
)

cc_library(
    name = "orchestration_macros_headers",
    hdrs = [
        "cpp/include/expose_internal.h",
        "cpp/include/expose_object_to_orchestration.h",
    ],
    includes = ["cpp/include"],
    visibility = ["//visibility:public"],
)
