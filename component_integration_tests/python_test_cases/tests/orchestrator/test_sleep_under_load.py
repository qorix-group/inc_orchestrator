import math
import os
from typing import Any

import pytest
from testing_utils import LogContainer

from component_integration_tests.python_test_cases.tests.cit_scenario import CitScenario


# Due to OS related condition variable wait behavior including scheduling, thread priority,
# hardware, load and other factors, sleep can spike and wait longer than expected.
# There is a bug filled for this topic: https://github.com/qorix-group/inc_orchestrator_internal/issues/142
def get_threshold_ms(expected_sleep_ms: int) -> int:
    """
    Calculate the threshold for sleep duration checks.
    """
    if expected_sleep_ms > 500:
        return math.ceil(expected_sleep_ms * 0.5)
    elif expected_sleep_ms > 100:
        return math.ceil(expected_sleep_ms * 1.5)
    else:
        return math.ceil(expected_sleep_ms * 5)


class TestShortSleepUnderLoad2W256Q(CitScenario):
    @pytest.fixture(scope="class")
    def scenario_name(self) -> str:
        return "orchestration.sleep.under_load"

    @pytest.fixture(scope="class")
    def test_config(self) -> dict[str, Any]:
        return {
            "runtime": {"workers": 2, "task_queue_size": 256},
            "test": {"sleep_duration_ms": 100, "run_count": 1, "cpu_load": "high"},
        }

    @pytest.fixture(scope="class")
    def execution_timeout(self) -> float:
        return 10.0

    def test_start_and_finish(self, logs_info_level: LogContainer):
        assert logs_info_level[0].message == "StartAction was executed", (
            "Program not started with first action in sequence"
        )
        assert logs_info_level[-1].message == "FinishAction was executed", (
            "Program not finished with last action in sequence"
        )

    @pytest.mark.parametrize(
        "sleep_name",
        ["Sleep1", "Sleep2", "Sleep3", "Sleep4", "Sleep5"],
    )
    def test_sleep_duration(
        self,
        logs_info_level: LogContainer,
        test_config: dict,
        sleep_name: str,
    ):
        [sleep_start, sleep_finish] = logs_info_level.get_logs(
            field="id",
            value=sleep_name,
        )
        sleep_duration_ms = (
            sleep_finish.timestamp - sleep_start.timestamp
        ).total_seconds() * 1000

        expected_sleep_ms = test_config["test"]["sleep_duration_ms"]

        threshold_ms = get_threshold_ms(expected_sleep_ms)
        assert (
            expected_sleep_ms <= sleep_duration_ms <= expected_sleep_ms + threshold_ms
        ), (
            f"Expected sleep duration {expected_sleep_ms} ms, "
            f"but got {sleep_duration_ms} ms. Threshold: {threshold_ms} ms."
        )


class TestMediumSleepUnderLoad2W256Q(TestShortSleepUnderLoad2W256Q):
    @pytest.fixture(scope="class")
    def test_config(self):
        return {
            "runtime": {"workers": 2, "task_queue_size": 256},
            "test": {"sleep_duration_ms": 500, "run_count": 1, "cpu_load": "high"},
        }

    @pytest.fixture(scope="class")
    def execution_timeout(self) -> float:
        return 10.0


class TestLongSleepUnderLoad2W256Q(TestMediumSleepUnderLoad2W256Q):
    @pytest.fixture(scope="class")
    def test_config(self):
        return {
            "runtime": {"workers": 2, "task_queue_size": 256},
            "test": {"sleep_duration_ms": 1000, "run_count": 1, "cpu_load": "high"},
        }

    @pytest.fixture(scope="class")
    def execution_timeout(self) -> float:
        return 10.0


@pytest.mark.do_not_repeat
@pytest.mark.only_nightly
class TestHugeAmountOfShortSleeps(TestShortSleepUnderLoad2W256Q):
    @pytest.fixture(scope="class")
    def test_config(self):
        return {
            "runtime": {"workers": 2, "task_queue_size": 256},
            "test": {"sleep_duration_ms": 5, "run_count": 2_000, "cpu_load": "low"},
        }

    @pytest.fixture(scope="class")
    def execution_timeout(self) -> float:
        return 120.0

    @pytest.mark.parametrize(
        "sleep_name",
        ["Sleep1", "Sleep2", "Sleep3", "Sleep4", "Sleep5"],
    )
    def test_sleep_duration(
        self,
        logs_info_level: LogContainer,
        test_config: dict,
        sleep_name: str,
    ):
        # Collect all start and finish logs for the given sleep_name
        [sleep_starts, sleep_finishes] = (
            logs_info_level.get_logs(field="id", value=sleep_name)
            .group_by("location")
            .values()
        )
        # Calculate duration of each sleep
        sleep_durations_ms = []
        for sleep_start, sleep_finish in zip(sleep_starts, sleep_finishes):
            sleep_durations_ms.append(
                (sleep_finish.timestamp - sleep_start.timestamp).total_seconds() * 1000
            )

        expected_sleep_ms = test_config["test"]["sleep_duration_ms"]
        threshold_ms = get_threshold_ms(expected_sleep_ms)

        # Check all sleep durations are within the expected range
        for sleep_duration_ms in sleep_durations_ms:
            assert (
                expected_sleep_ms
                <= sleep_duration_ms
                <= expected_sleep_ms + threshold_ms
            ), (
                f"Expected sleep duration {expected_sleep_ms} ms, "
                f"but got {sleep_duration_ms} ms. Threshold: {threshold_ms} ms."
            )
