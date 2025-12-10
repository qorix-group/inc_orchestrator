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
load("@score_virtualization//virtualization:virtualization_build.bzl", "virtualization_build")

virtualization_build(
    name = "virtualization",
    srcs = [
        "//src/orchestration:basic",
        "//src/orchestration:catch_error",
        "//src/orchestration:inter_process_event_receiver",
        "//src/orchestration:inter_process_event_sender",
    ],
    outs = [
        "custom/basic",
        "custom/catch_error",
        "custom/inter_process_event_receiver",
        "custom/inter_process_event_sender",
    ],
    disable = True,
    init_script = "custom_init.sh",
)
