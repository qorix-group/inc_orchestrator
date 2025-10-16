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
"""
Module contains function for determining current default TTL value.
"""

import platform
from socket import AF_INET, AF_INET6, AddressFamily
from subprocess import DEVNULL, PIPE, Popen


def _get_default_ttl_sysctl(address_family: AddressFamily) -> int:
    """
    Get current default TTL value using 'sysctl' command.

    Parameters
    ----------
    address_family : AddressFamily
        Address family - only 'AF_INET' and 'AF_INET6' supported.
    """
    # Select command based on address family.
    command = ["sysctl", "-n"]
    if address_family == AF_INET:
        command += ["net.ipv4.ip_default_ttl"]
    elif address_family == AF_INET6:
        command += ["net.ipv6.conf.default.hop_limit"]
    else:
        raise RuntimeError(f"Unsupported address family: {address_family.name}")

    # Run 'sysctl' command.
    with Popen(command, stdout=PIPE, stderr=DEVNULL, text=True) as p:
        stdout, _ = p.communicate()

    # Process and return output.
    return int(stdout.strip())


def get_default_ttl(address_family: AddressFamily = AF_INET) -> int:
    """
    Get current default TTL value.

    Parameters
    ----------
    address_family : AddressFamily
        Address family - only 'AF_INET' and 'AF_INET6' supported.
    """
    current_system = platform.system()
    match current_system:
        case "Linux":
            return _get_default_ttl_sysctl(address_family)
        case _:
            raise RuntimeError(f"System not supported: {current_system}")
