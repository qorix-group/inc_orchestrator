load("@crates//:defs.bzl", "all_crate_deps")
load("@rules_rust//rust:defs.bzl", "rust_binary")

rust_binary(
    name = "rust_test_scenarios",
    srcs = glob(["src/**/*.rs"]),
    tags = [
        "manual",
    ],
    visibility = ["//visibility:public"],
    deps = [
        "//src/async_runtime:libasync_runtime",
        "//src/foundation:libfoundation",
        "//src/orchestration:liborchestration",
    ] + all_crate_deps(
        normal = True,
    ),
)
