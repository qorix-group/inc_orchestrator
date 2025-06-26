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

_COMMON_DEPS = [
    "//src/foundation:libfoundation",
    "//src/async_runtime:libasync_runtime",
    "//src/logging_tracing:liblogging_tracing",
] + all_crate_deps(
    normal = True,
)

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
        "runtime-api-mock",
        "iceoryx-ipc",
    ],
    proc_macro_deps = [
        "//src/testing_macros:test_macro_utils",
    ],
    deps = [
        "//src/async_runtime:libasync_runtime_mocked",
        "//src/foundation:libfoundation",
        "//src/logging_tracing:liblogging_tracing",
        "//src/testing:test_utils",
    ] + all_crate_deps(
        normal = True,
    ),
)

rust_binary(
    name = "basic",
    srcs = [
        "examples/basic.rs",
    ] + glob(["examples/common/**/*.rs"]),
    visibility = ["//visibility:public"],
    deps = _COMMON_DEPS + [
        ":liborchestration",
    ],
)
