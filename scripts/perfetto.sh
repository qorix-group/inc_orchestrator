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

perfetto --txt -c logging_tracing/configs/perfetto.cfg -o "system_trace_$(date +"%Y-%m-%d_%H-%M-%S").txt"
