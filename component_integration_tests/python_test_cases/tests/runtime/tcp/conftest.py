from typing import Any

import pytest
from testing_utils.net import Address, create_connection

from component_integration_tests.python_test_cases.tests.cit_runtime_scenario import (
    Executable,
)


@pytest.fixture(scope="function")
def client_connection(connection_params: dict[str, Any], executable: Executable):
    """
    Create a TCP connection to the server.

    Parameters
    ----------
    connection : dict[str, Any]
        A dictionary containing the connection details with keys 'ip' and 'port'.
    executable : Executable
        The executable instance with running server.
    """

    executable.wait_for_log(
        lambda log_container: log_container.find_log(
            "message",
            pattern=f"TCP server listening on {connection_params['ip']}:{connection_params['port']}",
        )
        is not None
    )

    s = create_connection(Address.from_dict(connection_params), timeout=3.0)
    yield s
    s.close()
