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
load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_library", "rust_test")

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
    ":liborchestration",
]

rust_library(
    name = "liborchestration",
    srcs = glob(["src/**/*.rs"]),
    crate_features = ["iceoryx-ipc"],
    crate_name = "orchestration",
    visibility = ["//visibility:public"],
    deps = _COMMON_DEPS,
)

# manually pick files and deps as we need to run it with different runtime linked
rust_test(
    name = "tests",
    srcs = glob(["src/**/*.rs"]),
    crate_features = [
        "iceoryx-ipc",
    ],
    proc_macro_deps = [
        "//src/testing_macros:test_macro_utils",
    ],
    deps = _BASE_DEPS + [
        "//src/async_runtime:libasync_runtime_mocked",
        "//src/testing:test_utils",
    ],
)

rust_binary(
    name = "basic",
    srcs = [
        "examples/basic.rs",
    ] + glob(["examples/common/**/*.rs"]),
    visibility = ["//visibility:public"],
    deps = _EXAMPLE_DEPS,
)

rust_binary(
    name = "catch_error",
    srcs = [
        "examples/catch_error.rs",
    ] + glob(["examples/common/**/*.rs"]),
    visibility = ["//visibility:public"],
    deps = _EXAMPLE_DEPS,
)

rust_binary(
    name = "events_across_local_programs",
    srcs = [
        "examples/events_across_local_programs.rs",
    ] + glob(["examples/common/**/*.rs"]),
    visibility = ["//visibility:public"],
    deps = _EXAMPLE_DEPS,
)

rust_binary(
    name = "camera_drv_object_det",
    srcs = [
        "examples/camera_drv_object_det/camera_driver.rs",
        "examples/camera_drv_object_det/main.rs",
        "examples/camera_drv_object_det/object_detection.rs",
    ] + glob(["examples/common/**/*.rs"]),
    visibility = ["//visibility:public"],
    deps = _EXAMPLE_DEPS,
)
