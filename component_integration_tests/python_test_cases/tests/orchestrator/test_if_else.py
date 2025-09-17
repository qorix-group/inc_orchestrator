import json
from typing import Any

import pytest
from testing_utils import LogContainer

from component_integration_tests.python_test_cases.tests.cit_scenario import CitScenario


class TestBasicIfElseCondition(CitScenario):
    @pytest.fixture(scope="class")
    def scenario_name(self) -> str:
        return "orchestration.basic_if_else"

    @pytest.fixture(scope="class", params=[True, False])
    def condition(self, request: pytest.FixtureRequest) -> bool:
        return request.param

    @pytest.fixture(scope="class")
    def test_config(self, condition: bool) -> dict[str, Any]:
        return {
            "runtime": {"task_queue_size": 256, "workers": 4},
            "test": {"condition": condition},
        }

    def test_execution_branch(self, condition, logs_info_level: LogContainer):
        assert len(logs_info_level) == 1, "Expected exactly one log message"

        condition = json.dumps(condition)
        assert logs_info_level.contains_log(field="id", value=condition), (
            f"Expected execution of task with id={condition}"
        )


class TestNestedIfElseCondition(CitScenario):
    @pytest.fixture(scope="class")
    def scenario_name(self) -> str:
        return "orchestration.nested_if_else"

    @pytest.fixture(scope="class", params=[True, False])
    def outer_condition(self, request: pytest.FixtureRequest) -> bool:
        return request.param

    @pytest.fixture(scope="class", params=[True, False])
    def inner_condition(self, request: pytest.FixtureRequest) -> bool:
        return request.param

    @pytest.fixture(scope="class")
    def test_config(
        self, outer_condition: bool, inner_condition: bool
    ) -> dict[str, Any]:
        return {
            "runtime": {"task_queue_size": 256, "workers": 4},
            "test": {
                "outer_condition": outer_condition,
                "inner_condition": inner_condition,
            },
        }

    def test_execution_branch(
        self, outer_condition, inner_condition, logs_info_level: LogContainer
    ):
        assert len(logs_info_level) == 1, "Expected exactly one log message"

        outer_condition = json.dumps(outer_condition)
        inner_condition = json.dumps(inner_condition)
        expected_id = f"{outer_condition}_{inner_condition}"
        assert logs_info_level.contains_log(field="id", value=expected_id), (
            f"Expected execution of task with id={expected_id}"
        )
