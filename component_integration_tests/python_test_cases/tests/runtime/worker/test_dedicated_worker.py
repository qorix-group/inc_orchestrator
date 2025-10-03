from typing import Any
import pytest
from component_integration_tests.python_test_cases.tests.cit_scenario import CitScenario
from component_integration_tests.python_test_cases.tests.result_code import ResultCode
from testing_utils import ScenarioResult, LogContainer


class TestOnlyDedicatedWorkers(CitScenario):
    @pytest.fixture(scope="class")
    def scenario_name(self) -> str:
        return "runtime.worker.dedicated_worker.only_dedicated_workers"

    @pytest.fixture(scope="class", params=[1, 10])
    def num_dedicated(self, request: pytest.FixtureRequest) -> int:
        # Dedicated workers don't belong to regular workers pool.
        # - Run only one dedicated worker.
        # - Run more dedicated workers than regular workers available.
        return request.param

    @pytest.fixture(scope="class")
    def dedicated_workers(self, num_dedicated: int) -> list[dict[str, Any]]:
        result = []
        for i in range(num_dedicated):
            result.append({"id": f"dedicated_worker_{i}"})
        return result

    @pytest.fixture(scope="class")
    def test_config(self, dedicated_workers: list[dict[str, Any]]) -> dict[str, Any]:
        return {
            "runtime": {
                "task_queue_size": 256,
                "workers": 4,
                "dedicated_workers": dedicated_workers,
            }
        }

    def test_valid(
        self,
        logs_info_level: LogContainer,
        dedicated_workers: list[dict[str, Any]],
    ) -> None:
        # Check dedicated workers barrier wait result.
        wait_result_log = logs_info_level.find_log("wait_result")
        assert wait_result_log is not None
        assert wait_result_log.wait_result == "ok"

        # Check dedicated worker IDs.
        worker_logs = logs_info_level.get_logs("id", pattern="dedicated_worker_.*")
        act_worker_ids = set(log.id for log in worker_logs)
        exp_worker_ids = set(map(lambda x: x["id"], dedicated_workers))
        ids_diff = act_worker_ids.symmetric_difference(exp_worker_ids)
        assert not ids_diff, (
            f"Mismatch between worker IDs expected ({exp_worker_ids}) and actual ({act_worker_ids})"
        )


class TestSpawnToUnregisteredWorker(CitScenario):
    @pytest.fixture(scope="class")
    def scenario_name(self) -> str:
        return "runtime.worker.dedicated_worker.spawn_to_unregistered_worker"

    @pytest.fixture(scope="class")
    def test_config(self) -> dict[str, Any]:
        return {
            "runtime": {
                "task_queue_size": 256,
                "workers": 4,
            }
        }

    def capture_stderr(self) -> bool:
        return True

    def expect_command_failure(self) -> bool:
        return True

    def test_invalid(self, results: ScenarioResult) -> None:
        # Panic inside async causes 'SIGABRT'.
        # TODO: determine this should be panic, and not an error.
        assert results.return_code == ResultCode.SIGABRT

        assert results.stderr
        assert (
            "Tried to spawn on not registered dedicated worker UniqueWorkerId"
            in results.stderr
        )


class TestReregisterDedicatedWorker(CitScenario):
    @pytest.fixture(scope="class")
    def scenario_name(self) -> str:
        # Scenario is reused, should fail on init.
        return "runtime.worker.dedicated_worker.spawn_to_unregistered_worker"

    @pytest.fixture(scope="class")
    def test_config(self) -> dict[str, Any]:
        return {
            "runtime": {
                "task_queue_size": 256,
                "workers": 4,
                "dedicated_workers": [{"id": "same_id"}, {"id": "same_id"}],
            }
        }

    def capture_stderr(self) -> bool:
        return True

    def expect_command_failure(self) -> bool:
        return True

    def test_invalid(self, results: ScenarioResult) -> None:
        assert results.return_code == ResultCode.PANIC

        assert results.stderr
        assert "Cannot register same unique worker multiple times!" in results.stderr
