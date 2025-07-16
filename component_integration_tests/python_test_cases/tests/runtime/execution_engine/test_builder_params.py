from abc import abstractmethod
from typing import Any
import psutil
import pytest

from testing_utils import ScenarioResult
from cit_scenario import CitScenario


def _check_log(text: str, expected_line: str) -> bool:
    """
    Check if expected line occurred in provided text.
    """
    lines = text.strip().split("\n")
    return any(line == expected_line for line in lines)


# region task_queue_size


class TestTaskQueueSize(CitScenario):
    @pytest.fixture(scope="class")
    def scenario_name(self) -> str:
        return "basic.only_shutdown"

    @abstractmethod
    def queue_size(self, request: pytest.FixtureRequest) -> int:
        pass

    @pytest.fixture(scope="class")
    def test_config(self, queue_size: int) -> dict[str, Any]:
        return {"runtime": {"task_queue_size": queue_size, "workers": 1}}


class TestTaskQueueSize_Valid(TestTaskQueueSize):
    @pytest.fixture(scope="class", params=range(0, 32))
    def queue_size(self, request: pytest.FixtureRequest) -> int:
        exponent = request.param
        queue_size = 2**exponent

        # Prevent allocation of a queue too large for available memory.
        # Required for testing on smaller devices (e.g., CI runners).
        size_heuristic = queue_size * 8
        total_memory = psutil.virtual_memory().total + psutil.swap_memory().total
        if size_heuristic >= total_memory:
            pytest.xfail(
                reason=f"Requested queue size ({queue_size}) is too large for available memory ({total_memory})"
            )

        return queue_size

    def test_valid(self, results: ScenarioResult) -> None:
        assert results.return_code == 0


class TestTaskQueueSize_Invalid(TestTaskQueueSize):
    @pytest.fixture(scope="class", params=[0, 10, 321, 1234, 2**16 - 1, 2**32 - 1])
    def queue_size(self, request: pytest.FixtureRequest) -> int:
        return request.param

    def capture_stderr(self) -> bool:
        return True

    def test_invalid(self, results: ScenarioResult, queue_size: int) -> None:
        assert results.return_code == 101
        assert results.stderr is not None
        assert _check_log(
            results.stderr, f"Task queue size ({queue_size}) must be power of two"
        )


# endregion


# region workers


class TestWorkers(CitScenario):
    @pytest.fixture(scope="class")
    def scenario_name(self) -> str:
        return "basic.only_shutdown"

    @abstractmethod
    def workers(self, request: pytest.FixtureRequest) -> int:
        pass

    @pytest.fixture(scope="class")
    def test_config(self, workers: int) -> dict[str, Any]:
        return {"runtime": {"task_queue_size": 256, "workers": workers}}


class TestWorkers_Valid(TestWorkers):
    @pytest.fixture(scope="class", params=[1, 33, 100, 128])
    def workers(self, request: pytest.FixtureRequest) -> int:
        return request.param

    def test_valid(self, results: ScenarioResult) -> None:
        assert results.return_code == 0


class TestWorkers_Invalid(TestWorkers):
    @pytest.fixture(scope="class", params=[0, 129, 1000])
    def workers(self, request: pytest.FixtureRequest) -> int:
        return request.param

    def capture_stderr(self) -> bool:
        return True

    def test_invalid(self, results: ScenarioResult, workers: int) -> None:
        assert results.return_code == 101
        assert results.stderr is not None
        assert _check_log(
            results.stderr,
            f"Cannot create engine with {workers} workers. Min is 1 and max is 128",
        )


# endregion

# region thread_priority


class TestThreadPriority(CitScenario):
    @pytest.fixture(scope="class")
    def scenario_name(self) -> str:
        return "basic.only_shutdown"

    @pytest.fixture(scope="class", params=[0, 120, 255])
    def test_config(self, request: pytest.FixtureRequest) -> dict[str, Any]:
        return {
            "runtime": {
                "task_queue_size": 256,
                "workers": 1,
                "thread_priority": request.param,
            }
        }

    def test_valid(self, results: ScenarioResult) -> None:
        assert results.return_code == 0


# endregion

# region thread_affinity


class TestThreadAffinity(CitScenario):
    @pytest.fixture(scope="class")
    def scenario_name(self) -> str:
        return "basic.only_shutdown"

    @pytest.fixture(scope="class", params=[0, 2**16, 2**32, 2**48, 2**63])
    def test_config(self, request: pytest.FixtureRequest) -> dict[str, Any]:
        return {
            "runtime": {
                "task_queue_size": 256,
                "workers": 1,
                "thread_affinity": request.param,
            }
        }

    def test_valid(self, results: ScenarioResult) -> None:
        assert results.return_code == 0


# endregion

# region thread_stack_size


class TestThreadStackSize(CitScenario):
    @pytest.fixture(scope="class")
    def scenario_name(self) -> str:
        return "basic.only_shutdown"

    @abstractmethod
    def thread_stack_size(self, request: pytest.FixtureRequest) -> int:
        pass

    @pytest.fixture(scope="class")
    def test_config(self, thread_stack_size: int) -> dict[str, Any]:
        return {
            "runtime": {
                "task_queue_size": 256,
                "workers": 1,
                "thread_stack_size": thread_stack_size,
            }
        }


class TestThreadStackSize_Valid(TestThreadStackSize):
    @pytest.fixture(scope="class", params=[1024 * 128, 1024 * 1024])
    def thread_stack_size(self, request: pytest.FixtureRequest) -> int:
        return request.param

    def test_valid(self, results: ScenarioResult) -> None:
        assert results.return_code == 0


class TestThreadStackSize_TooSmall(TestThreadStackSize):
    @pytest.fixture(scope="class", params=[0, 1024 * 8])
    def thread_stack_size(self, request: pytest.FixtureRequest) -> int:
        return request.param

    def capture_stderr(self) -> bool:
        return True

    def test_invalid(self, results: ScenarioResult) -> None:
        assert results.return_code == -9
        assert results.stderr is not None
        assert _check_log(
            results.stderr,
            "called `Result::unwrap()` on an `Err` value: StackSizeTooSmall",
        )


# endregion
