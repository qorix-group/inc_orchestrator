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
load("@score_cr_checker//:cr_checker.bzl", "copyright_checker")
load("@score_dash_license_checker//:dash.bzl", "dash_license_checker")
load("@score_format_checker//:macros.bzl", "use_format_targets")
load("@score_python_basics//:defs.bzl", "score_virtualenv")
load("@score_starpls_lsp//:starpls.bzl", "setup_starpls")
load("//:project_config.bzl", "PROJECT_CONFIG")

setup_starpls(
    name = "starpls_server",
    visibility = ["//visibility:public"],
)

score_virtualenv(
    name = "ide_support",
    reqs = [],
    venv_name = ".venv",
)

copyright_checker(
    name = "copyright",
    srcs = [
        "src",
        "tests",
        "//:BUILD",
        "//:MODULE.bazel",
    ],
    config = "@score_cr_checker//resources:config",
    template = "@score_cr_checker//resources:templates",
    visibility = ["//visibility:public"],
)

## Add the cargo lock file once it is available
#dash_license_checker(
#    src = "//examples:cargo_lock",
#    file_type = "",  # let it auto-detect based on project_config
#    project_config = PROJECT_CONFIG,
#    visibility = ["//visibility:public"],
#)

# Add target for formatting checks
use_format_targets()

