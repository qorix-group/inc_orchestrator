from typing import Any

import pytest

from component_integration_tests.python_test_cases.tests.cit_runtime_scenario import (
    Executable,
    NetHelper,
)


@pytest.fixture(scope="function")
def client_connection(test_config: dict[str, Any], executable: Executable):
    """
    Create a TCP connection to the server.
    """

    connection_details = test_config.get("connection", {})
    executable.wait_for_log(
        f"TCP server listening on {connection_details['ip']}:{connection_details['port']}"
    )

    s = NetHelper.connection_builder(**connection_details, timeout=3.0)
    yield s
    s.close()
