import math
from typing import Any

import pytest
from testing_utils import LogContainer

from component_integration_tests.python_test_cases.tests.cit_scenario import (
    CitScenario,
)


class TestSleepDuration(CitScenario):
    @pytest.fixture(scope="class", params=[0, 10, 50, 100, 500, 1000, 3000], ids=lambda x: f"sleep_{x}ms")
    def sleep_duration_ms(self, request: pytest.FixtureRequest) -> int:
        return request.param

    @pytest.fixture(scope="class", params=[1, 4], ids=lambda x: f"workers_{x}")
    def workers(self, request: pytest.FixtureRequest) -> int:
        return request.param

    @pytest.fixture(scope="class")
    def scenario_name(self) -> str:
        return "runtime.sleep.basic"

    @pytest.fixture(scope="class")
    def sleep_count(self) -> int:
        return 10

    @pytest.fixture(scope="class")
    def non_blocking_sleep_tasks(self, sleep_duration_ms: int, sleep_count: int) -> list[dict[str, Any]]:
        return [{"id": f"non_blocking_sleep_{x}", "delay_ms": sleep_duration_ms} for x in range(1, sleep_count + 1)]

    @pytest.fixture(scope="class")
    def test_config(self, workers: int, non_blocking_sleep_tasks: list[dict[str, Any]]) -> dict[str, Any]:
        return {
            "runtime": {"workers": workers, "task_queue_size": 256},
            "test": {
                "non_blocking_sleep_tasks": non_blocking_sleep_tasks,
                "blocking_sleep_tasks": [],
                "non_sleep_tasks": [],
            },
        }

    def test_completeness(self, non_blocking_sleep_tasks: list[dict[str, Any]], logs_info_level: LogContainer):
        for task in non_blocking_sleep_tasks:
            task_id = task["id"]
            entries = logs_info_level.get_logs(field="id", value=task_id)

            assert len(entries) == 2, f"Expected two entries for the task {task_id}."
            assert entries[0].location == "begin"
            assert entries[1].location == "end"

    def test_sleep_duration_strict(
        self, non_blocking_sleep_tasks: list[dict[str, Any]], sleep_duration_ms: int, logs_info_level: LogContainer
    ):
        for task in non_blocking_sleep_tasks:
            (begin_entry, end_entry) = logs_info_level.get_logs(field="id", value=task["id"])
            sleep_result_ms = (end_entry.timestamp - begin_entry.timestamp).total_seconds() * 1000

            expected_sleep_ms = sleep_duration_ms
            assert sleep_result_ms >= expected_sleep_ms, (
                f"Sleep duration for task {task['id']} is less than minimum expected."
            )


class TestMultipleSleepsWithRegularTasks(CitScenario):
    @pytest.fixture(scope="class")
    def scenario_name(self) -> str:
        return "runtime.sleep.basic"

    @pytest.fixture(scope="class", params=[1, 4], ids=lambda x: f"workers_{x}")
    def workers(self, request: pytest.FixtureRequest) -> int:
        return request.param

    @pytest.fixture(scope="class")
    def non_blocking_sleep_tasks(self) -> list[dict[str, Any]]:
        return [
            {"id": "non_blocking_sleep_1", "delay_ms": 3000},
            {"id": "non_blocking_sleep_2", "delay_ms": 2500},
            {"id": "non_blocking_sleep_3", "delay_ms": 2000},
            {"id": "non_blocking_sleep_4", "delay_ms": 1500},
            {"id": "non_blocking_sleep_5", "delay_ms": 1000},
            {"id": "non_blocking_sleep_6", "delay_ms": 500},
            {"id": "non_blocking_sleep_7", "delay_ms": 100},
        ]

    @pytest.fixture(scope="class")
    def non_sleep_tasks(self) -> list[str]:
        return [f"non_sleep_{x}" for x in range(1, 11)]

    @pytest.fixture(scope="class")
    def test_config(
        self, workers: int, non_blocking_sleep_tasks: list[dict[str, Any]], non_sleep_tasks: list[str]
    ) -> dict[str, Any]:
        return {
            "runtime": {"workers": workers, "task_queue_size": 256},
            "test": {
                "non_blocking_sleep_tasks": non_blocking_sleep_tasks,
                "blocking_sleep_tasks": [],
                "non_sleep_tasks": non_sleep_tasks,
            },
        }

    def test_task_completeness(
        self,
        non_blocking_sleep_tasks: list[dict[str, Any]],
        non_sleep_tasks: list[str],
        logs_info_level: LogContainer,
    ):
        # non_blocking_sleep_entries = logs_info_level.get_logs(field="id", pattern="non_blocking_sleep*")
        non_blocking_sleep_group = logs_info_level.get_logs(field="id", pattern="non_blocking_sleep*").group_by(
            "location"
        )
        assert len(non_blocking_sleep_group["begin"]) == len(non_blocking_sleep_tasks), (
            "Not all non-blocking sleep tasks started."
        )
        assert len(non_blocking_sleep_group["end"]) == len(non_blocking_sleep_tasks), (
            "Not all non-blocking sleep tasks finished."
        )

        non_sleep_entries = logs_info_level.get_logs(field="id", pattern="non_sleep*")
        assert len(non_sleep_entries) == len(non_sleep_tasks), "Not all non-sleep tasks executed."

    def test_sleep_duration(self, non_blocking_sleep_tasks: list[dict[str, Any]], logs_info_level: LogContainer):
        for task in non_blocking_sleep_tasks:
            entries = logs_info_level.get_logs(field="id", value=task["id"])
            assert len(entries) == 2, "Expected two entries for the task."

            (begin_entry, end_entry) = entries
            expected_sleep_ms = task["delay_ms"]

            sleep_result_ms = (end_entry.timestamp - begin_entry.timestamp).total_seconds() * 1000
            assert expected_sleep_ms <= sleep_result_ms, f"Sleep finished too early for task {task['id']}."


class TestMultipleSleepsWithBlockedWorkers(CitScenario):
    @pytest.fixture(scope="class")
    def scenario_name(self) -> str:
        return "runtime.sleep.basic"

    @pytest.fixture(scope="class")
    def non_blocking_sleep_tasks(self) -> list[dict[str, Any]]:
        return [
            {"id": "non_blocking_sleep_1", "delay_ms": 2000},
            {"id": "non_blocking_sleep_2", "delay_ms": 1500},
            {"id": "non_blocking_sleep_3", "delay_ms": 1200},
            {"id": "non_blocking_sleep_4", "delay_ms": 1000},
            {"id": "non_blocking_sleep_5", "delay_ms": 700},
            {"id": "non_blocking_sleep_6", "delay_ms": 500},
            {"id": "non_blocking_sleep_7", "delay_ms": 300},
        ]

    @pytest.fixture(scope="class")
    def blocking_sleep_tasks(self) -> list[dict[str, Any]]:
        return [
            {"id": "blocking_sleep_1", "delay_ms": 3300},
            {"id": "blocking_sleep_2", "delay_ms": 3300},
            {"id": "blocking_sleep_3", "delay_ms": 3300},
        ]

    @pytest.fixture(scope="class")
    def non_sleep_tasks(self) -> list[str]:
        return [f"non_sleep_{x}" for x in range(1, 11)]

    @pytest.fixture(scope="class")
    def test_config(
        self,
        non_blocking_sleep_tasks: list[dict[str, Any]],
        blocking_sleep_tasks: list[dict[str, Any]],
        non_sleep_tasks: list[str],
    ) -> dict[str, Any]:
        return {
            "runtime": {"workers": 4, "task_queue_size": 256},
            "test": {
                "non_blocking_sleep_tasks": non_blocking_sleep_tasks,
                "blocking_sleep_tasks": blocking_sleep_tasks,
                "non_sleep_tasks": non_sleep_tasks,
            },
        }

    def test_task_completeness(
        self,
        non_blocking_sleep_tasks: list[dict[str, Any]],
        blocking_sleep_tasks: list[dict[str, Any]],
        non_sleep_tasks: list[str],
        logs_info_level: LogContainer,
    ):
        non_blocking_sleep_entries = logs_info_level.get_logs(field="id", pattern="non_blocking_sleep*")
        expected_non_blocking_count = len(non_blocking_sleep_tasks) * 2  # 2 for begin and end
        assert len(non_blocking_sleep_entries) == expected_non_blocking_count, (
            "Not all non-blocking sleep tasks executed."
        )

        blocking_sleep_entries = logs_info_level.get_logs(field="id", pattern="^blocking_sleep*")
        expected_blocking_count = len(blocking_sleep_tasks) * 2  # 2 for begin and end
        assert len(blocking_sleep_entries) == expected_blocking_count, "Not all blocking sleep tasks executed."

        non_sleep_entries = logs_info_level.get_logs(field="id", pattern="non_sleep*")
        assert len(non_sleep_entries) == len(non_sleep_tasks), "Not all non-sleep tasks executed."

    def test_sleep_duration(self, non_blocking_sleep_tasks: list[dict[str, Any]], logs_info_level: LogContainer):
        for task in non_blocking_sleep_tasks:
            entries = logs_info_level.get_logs(field="id", value=task["id"])
            assert len(entries) == 2, "Expected two entries for the task."

            (begin_entry, end_entry) = entries
            expected_sleep_ms = task["delay_ms"]

            sleep_result_ms = (end_entry.timestamp - begin_entry.timestamp).total_seconds() * 1000
            assert expected_sleep_ms <= sleep_result_ms, f"Sleep finished too early for task {task['id']}."
