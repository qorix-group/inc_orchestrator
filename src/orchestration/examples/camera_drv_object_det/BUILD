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
load("@crates//:defs.bzl", "all_crate_deps")
load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_library")

_BASE_DEPS = [
    "//src/foundation:libfoundation",
    "//src/logging_tracing:liblogging_tracing",
] + all_crate_deps(
    normal = True,
)

_COMMON_DEPS = _BASE_DEPS + [
    "//src/async_runtime:libasync_runtime",
]

_EXAMPLE_DEPS = _COMMON_DEPS + [
    "//src/orchestration:liborchestration",
]

rust_binary(
    name = "camera_drv_object_det",
    srcs = [
        "src/camera_driver.rs",
        "src/main.rs",
        "src/object_detection.rs",
    ],
    proc_macro_deps = [
        "//src/orchestration_macros:orch_macros",
    ],
    visibility = ["//visibility:public"],
    deps = _EXAMPLE_DEPS + [
        ":libobject_detection",
    ],
)

cc_library(
    name = "libobject_detection",
    srcs = [
        "cpp/object_detection.cpp",
    ],
    hdrs = [
        "cpp/include/object_detection.h",
    ],
    includes = [
        "cpp",
    ],
    deps = [
        "//src/orchestration_macros:orchestration_macros_headers",
    ],
)
