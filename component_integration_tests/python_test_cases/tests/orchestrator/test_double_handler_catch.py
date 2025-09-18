from typing import Any

import pytest
from testing_utils import ScenarioResult

from component_integration_tests.python_test_cases.tests.cit_scenario import (
    CitScenario,
    ResultCode,
)


class TestDoubleSameCatchHandler(CitScenario):
    @pytest.fixture(scope="class")
    def scenario_name(self) -> str:
        return "orchestration.catch.double_same_handler_error"

    @pytest.fixture(scope="class")
    def test_config(self) -> dict[str, Any]:
        return {"runtime": {"task_queue_size": 256, "workers": 4}}

    def capture_stderr(self) -> bool:
        return True

    def expect_command_failure(self) -> bool:
        return True

    def test_double_handler_panic(self, results: ScenarioResult):
        assert results.return_code == ResultCode.PANIC
        assert results.stderr
        assert "Catch: Cannot set handler multiple times" in results.stderr


class TestDoubleDiffCatchHandler(TestDoubleSameCatchHandler):
    @pytest.fixture(scope="class")
    def scenario_name(self) -> str:
        return "orchestration.catch.double_diff_handler_error"
