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

load("@score_toolchains_qnx//rules/fs:ifs.bzl", "qnx_ifs")

config_setting(
    name = "is_qnx_x86_64",
    constraint_values = [
        "@platforms//cpu:x86_64",
        "@platforms//os:qnx",
    ],
)

config_setting(
    name = "is_qnx_aarch64",
    constraint_values = [
        "@platforms//cpu:aarch64",
        "@platforms//os:qnx",
    ],
)

# Here You can install additional artefacts into the initramfs tree
genrule(
    name = "stage_build_artefacts",
    srcs = [
        "//src/orchestration:basic",
        "//src/orchestration:catch_error",
        "//src/orchestration:inter_process_event_sender",
        "//src/orchestration:inter_process_event_receiver",
    ],
    outs = [
        "custom/basic",
        "custom/catch_error",
        "custom/inter_process_event_sender",
        "custom/inter_process_event_receiver",
    ],
    cmd = """
      set -e
      srcs=($(SRCS))
      outs=($(OUTS))

      for i in $${!srcs[@]}; do
        out=$${outs[$$i]}
        mkdir -p "$$(dirname "$$out")"
        cp "$${srcs[$$i]}" "$$out"
        chmod +x "$$out"
      done
    """,
    visibility = ["//visibility:public"],
)

filegroup(
    name = "overlay_tree",
    srcs = glob(["install/**"]),
    visibility = ["//visibility:public"],
)

qnx_ifs(
    name = "init",
    # Only include the install tree as inputs on aarch64
    srcs = select({
        ":is_qnx_aarch64": [
            ":overlay_tree",
            ":stage_build_artefacts",
        ],
        "//conditions:default": [],
    }),
    out = select({
        ":is_qnx_x86_64": "init_x86_64.ifs",
        ":is_qnx_aarch64": "init_aarch64.ifs",
    }),
    build_file = select({
        ":is_qnx_x86_64": "init_x86_64.build",
        ":is_qnx_aarch64": "init_rpi4.build",
    }),

    # Only pass -r on aarch64 (mkifs will receive: -r install)
    search_roots = select({
        ":is_qnx_aarch64": ["install"],  # relative to the .build file’s dir
        "//conditions:default": [],
    }),
)

sh_binary(
    name = "run_qemu",
    srcs = ["run_qemu.sh"],
    args = [
        "$(location @toolchains_qnx_sdp//:host_dir)",
        "$(location :init)",
        "$(location @custom_qemu//:qemu_bin)",
        "$(location @custom_qemu//:rpi4_dtb)",
    ],
    data = [
        ":init",
        "@custom_qemu//:qemu_bin",
        "@custom_qemu//:rpi4_dtb",
        "@toolchains_qnx_sdp//:host_all",
        "@toolchains_qnx_sdp//:host_dir",
    ],
)
