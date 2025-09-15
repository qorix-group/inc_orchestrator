import socket
from typing import Any

import pytest

from component_integration_tests.python_test_cases.tests.cit_scenario import (
    CitContinousScenario,
    NetHelper,
)


class TestTcpServer(CitContinousScenario):
    @pytest.fixture(scope="class")
    def scenario_name(self) -> str:
        return "runtime.tcp.basic_server"

    @pytest.fixture(scope="class")
    def test_config(self) -> dict[str, Any]:
        return {
            "runtime": {"task_queue_size": 256, "workers": 4},
            "connection": {"ip": "127.0.0.1", "port": 7878},
        }

    def test_tcp_echo(self, connection: socket.socket) -> None:
        message = b"Echo!"
        connection.sendall(message)
        data = connection.recv(1024)
        assert message == data

    def test_multiple_connections(self, test_config: dict[str, Any]) -> None:
        connection_details = test_config.get("connection", {})
        conn1 = NetHelper.connection_builder(**connection_details)
        conn2 = NetHelper.connection_builder(**connection_details)
        conn3 = NetHelper.connection_builder(**connection_details)

        with conn1, conn2, conn3:
            msg1 = b"Uno"
            msg2 = b"Dos"
            msg3 = b"Tres"

            conn1.sendall(msg1)
            conn2.sendall(msg2)
            conn3.sendall(msg3)

            data1 = conn1.recv(1024)
            data2 = conn2.recv(1024)
            data3 = conn3.recv(1024)

            assert msg1 == data1
            assert msg2 == data2
            assert msg3 == data3
