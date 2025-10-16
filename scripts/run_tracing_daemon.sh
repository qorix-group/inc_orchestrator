#!/bin/bash
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

# We assume traced and all is in a path, like export PATH="WHATEVER_/perfetto/out/linux/:$PATH"
set -e



pkill -9 -f traced || true
pkill -9 -f traced_probes || true

if [ ! -w /sys/kernel/tracing ]; then
    echo "tracefs not accessible, try sudo chown -R $USER /sys/kernel/tracing"
    sudo chown -R "$USER" /sys/kernel/tracing
fi

echo 0 > /sys/kernel/tracing/tracing_on

traced &
traced_probes &
