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

load("@bazel_skylib//rules:common_settings.bzl", "bool_flag")

bool_flag(
    name = "k_runtime_disable_time_correction",
    build_setting_default = False,
)

# Explicit time correction disabling needed for Linux VMs on Windows. As this applies to all
# x86_64 it may be not needed for native Linux on x86_64
config_setting(
    name = "runtime_disable_time_correction",
    flag_values = {
        ":k_runtime_disable_time_correction": "True",
    },
    visibility = ["//visibility:public"],
)
