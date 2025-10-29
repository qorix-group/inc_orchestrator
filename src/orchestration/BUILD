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
load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_library", "rust_test")
load("@score_orchestrator_crates//:defs.bzl", "all_crate_deps")

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
    crate_features = [
        "iceoryx-ipc",
        "orch_tracing",
    ],
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
        "orch_tracing",
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
    name = "main_macro_basic",
    srcs = [
        "examples/main_macro_basic.rs",
    ] + glob(["examples/common/**/*.rs"]),
    proc_macro_deps = [
        "//src/async_runtime_macros:runtime_macros",
    ],
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
    name = "inter_process_event_sender",
    srcs = [
        "examples/inter_process_event/sender.rs",
    ] + glob(["examples/common/**/*.rs"]),
    visibility = ["//visibility:public"],
    deps = _EXAMPLE_DEPS,
    crate_root = "examples/inter_process_event/sender.rs",
)

rust_binary(
    name = "inter_process_event_receiver",
    srcs = [
        "examples/inter_process_event/receiver.rs",
    ] + glob(["examples/common/**/*.rs"]),
    crate_root = "examples/inter_process_event/receiver.rs",
    visibility = ["//visibility:public"],
    deps = _EXAMPLE_DEPS,
)
