import pytest
from testing_tools.log_container import LogContainer


class TestSingleConcurrency1W2Q:
    @pytest.fixture(scope="class")
    def scenario_name(self):
        return "orchestration.single_concurrency"

    @pytest.fixture(scope="class")
    def test_config(self):
        return {"runtime": {"task_queue_size": 2, "workers": 1}}

    def test_each_function_standalone_order(self, test_results: LogContainer):
        for function in ["Function1", "Function2", "Function3"]:
            expected_order = [
                f"Start of '{function}' function",
                f"End of '{function}' function",
            ]
            execution_order = [
                log.message
                for log in test_results.get_logs_by_field(
                    field="message", pattern=function
                )
            ]

            assert execution_order == expected_order, (
                f"Execution order for {function} is incorrect"
            )

    def test_concurrency_finished_before_final_task(self, test_results: LogContainer):
        # Check if all branches in Concurrency were executed before next Sequence step
        finish_action = test_results.find_log(
            field="message", pattern="FinishAction was executed"
        )

        end_fun_1 = test_results.find_log(
            field="message", pattern="End of 'Function1' function"
        )
        assert end_fun_1.timestamp < finish_action.timestamp, (
            "Function1 execution should be finished before FinishAction was executed"
        )
        end_fun_2 = test_results.find_log(
            field="message", pattern="End of 'Function2' function"
        )
        assert end_fun_2.timestamp < finish_action.timestamp, (
            "Function2 execution should be finished before FinishAction was executed"
        )
        end_fun_3 = test_results.find_log(
            field="message", pattern="End of 'Function3' function"
        )

        assert end_fun_3.timestamp < finish_action.timestamp, (
            "Function3 execution should be finished before FinishAction was executed"
        )


class TestSingleConcurrency2W2Q(TestSingleConcurrency1W2Q):
    @pytest.fixture(scope="class")
    def test_config(self):
        return {"runtime": {"task_queue_size": 2, "workers": 2}}


class TestMultipleConcurrency1W2Q:
    @pytest.fixture(scope="class")
    def scenario_name(self):
        return "orchestration.multiple_concurrency"

    @pytest.fixture(scope="class")
    def test_config(self):
        return {"runtime": {"task_queue_size": 2, "workers": 1}}

    def test_concurrency1_finished_before_final_task(self, test_results: LogContainer):
        # Check if all branches in Concurrency1 were executed before FinishAction
        finish_action = test_results.find_log(
            field="message", pattern="FinishAction was executed"
        )

        end_fun_1 = test_results.find_log(
            field="message", pattern="End of 'Function1' function"
        )
        assert end_fun_1.timestamp < finish_action.timestamp, (
            "Function1 execution should be finished before FinishAction was executed"
        )
        end_fun_2 = test_results.find_log(
            field="message", pattern="End of 'Function2' function"
        )
        assert end_fun_2.timestamp < finish_action.timestamp, (
            "Function2 execution should be finished before FinishAction was executed"
        )
        end_fun_3 = test_results.find_log(
            field="message", pattern="End of 'Function3' function"
        )

        assert end_fun_3.timestamp < finish_action.timestamp, (
            "Function3 execution should be finished before FinishAction was executed"
        )

    def test_concurrency1_finished_before_concurrency2_start(
        self, test_results: LogContainer
    ):
        # Check if all branches in Concurrency1 were executed before starting Concurrency2
        # Find all functions starts from Concurrency2 and pick the first one by timestamp
        [*functions_starts_from_concurrency2] = test_results.get_logs_by_field(
            field="message", pattern="Start of 'Function(4|5|6)' function"
        )
        concurrency2_start = functions_starts_from_concurrency2[0]

        end_fun_1 = test_results.find_log(
            field="message", pattern="End of 'Function1' function"
        )
        assert end_fun_1.timestamp < concurrency2_start.timestamp, (
            "Function1 execution from Concurrency1 block should be finished before Concurrency2 was started"
        )
        end_fun_2 = test_results.find_log(
            field="message", pattern="End of 'Function2' function"
        )
        assert end_fun_2.timestamp < concurrency2_start.timestamp, (
            "Function2 execution from Concurrency1 block should be finished before Concurrency2 was started"
        )
        end_fun_3 = test_results.find_log(
            field="message", pattern="End of 'Function3' function"
        )

        assert end_fun_3.timestamp < concurrency2_start.timestamp, (
            "Function3 execution from Concurrency1 block should be finished before Concurrency2 was started"
        )

    def test_concurrency2_finished_before_final_task(self, test_results: LogContainer):
        # Check if all branches in Concurrency2 were executed before FinishAction
        finish_action = test_results.find_log(
            field="message", pattern="FinishAction was executed"
        )

        end_fun_4 = test_results.find_log(
            field="message", pattern="End of 'Function4' function"
        )
        assert end_fun_4.timestamp < finish_action.timestamp, (
            "Function4 execution should be finished before FinishAction was executed"
        )
        end_fun_5 = test_results.find_log(
            field="message", pattern="End of 'Function5' function"
        )
        assert end_fun_5.timestamp < finish_action.timestamp, (
            "Function5 execution should be finished before FinishAction was executed"
        )
        end_fun_6 = test_results.find_log(
            field="message", pattern="End of 'Function6' function"
        )

        assert end_fun_6.timestamp < finish_action.timestamp, (
            "Function6 execution should be finished before FinishAction was executed"
        )

    def test_concurrency1_execution_order(self, test_results: LogContainer):
        # Find all functions starts from Concurrency1 and pick the first one by timestamp
        [*functions_starts_from_concurrency1] = test_results.get_logs_by_field(
            field="message", pattern="Start of 'Function(1|2|3)' function"
        )
        concurrency1_start = functions_starts_from_concurrency1[0]

        # Find all functions ends from Concurrency1 and pick the last one by timestamp
        [*functions_ends_from_concurrency1] = test_results.get_logs_by_field(
            field="message", pattern="End of 'Function(1|2|3)' function"
        )
        concurrency1_end = functions_ends_from_concurrency1[-1]

        # IntermediateAction should be executed after Concurrency1
        intermediate_action = test_results.find_log(
            field="message", pattern="IntermediateAction was executed"
        )

        assert (
            concurrency1_start.timestamp
            < concurrency1_end.timestamp
            < intermediate_action.timestamp
        ), (
            "Incorrect execution order. Expected Concurrency1 to finish before IntermediateAction"
        )

    def test_concurrency2_execution_order(self, test_results: LogContainer):
        # IntermediateAction should be executed after Concurrency1
        intermediate_action = test_results.find_log(
            field="message", pattern="IntermediateAction was executed"
        )
        # Find all functions starts from Concurrency2 and pick the first one by timestamp
        [*functions_starts_from_concurrency2] = test_results.get_logs_by_field(
            field="message", pattern="Start of 'Function(4|5|6)' function"
        )
        concurrency2_start = functions_starts_from_concurrency2[0]
        # Find all functions ends from Concurrency2 and pick the last one by timestamp
        [*functions_ends_from_concurrency2] = test_results.get_logs_by_field(
            field="message", pattern="End of 'Function(4|5|6)' function"
        )
        concurrency2_end = functions_ends_from_concurrency2[-1]

        # FinishActrion should be executed after Concurrency2
        finish_action = test_results.find_log(
            field="message", pattern="FinishAction was executed"
        )

        assert (
            intermediate_action.timestamp
            < concurrency2_start.timestamp
            < concurrency2_end.timestamp
            < finish_action.timestamp
        ), (
            "Incorrect execution order. Expected Concurrency2 to start after IntermediateAction and finish before FinishAction"
        )


class TestMultipleConcurrency2W2Q(TestMultipleConcurrency1W2Q):
    @pytest.fixture(scope="class")
    def test_config(self):
        return {"runtime": {"task_queue_size": 2, "workers": 2}}


class TestMultipleConcurrency5W256Q(TestMultipleConcurrency1W2Q):
    @pytest.fixture(scope="class")
    def test_config(self):
        return {"runtime": {"task_queue_size": 256, "workers": 5}}


class TestNestedConcurrency1W2Q:
    @pytest.fixture(scope="class")
    def scenario_name(self):
        return "orchestration.nested_concurrency"

    @pytest.fixture(scope="class")
    def test_config(self):
        return {"runtime": {"task_queue_size": 2, "workers": 1}}

    def test_inner_concurrency_finished_before_final_task(
        self, test_results: LogContainer
    ):
        # Check if all branches in InnerConcurrency were executed before FinishAction
        finish_action = test_results.find_log(
            field="message", pattern="FinishAction was executed"
        )

        [*functions_end_from_inner_concurrency] = test_results.get_logs_by_field(
            field="message", pattern="End of 'InnerFunction.*' function"
        )
        inner_concurrency_end = functions_end_from_inner_concurrency[-1]
        assert inner_concurrency_end.timestamp < finish_action.timestamp, (
            "Inner concurrency execution should be finished before FinishAction was executed"
        )

    def test_outer_concurrency_finished_before_final_task(
        self, test_results: LogContainer
    ):
        # Check if all branches in OuterConcurrency were executed before FinishAction
        finish_action = test_results.find_log(
            field="message", pattern="FinishAction was executed"
        )

        [*functions_end_from_outer_concurrency] = test_results.get_logs_by_field(
            field="message", pattern="End of 'OuterFunction.*' function"
        )
        outer_concurrency_end = functions_end_from_outer_concurrency[-1]
        assert outer_concurrency_end.timestamp < finish_action.timestamp, (
            "Outer concurrency execution should be finished before FinishAction was executed"
        )


class TestNestedConcurrency2W2Q(TestNestedConcurrency1W2Q):
    @pytest.fixture(scope="class")
    def test_config(self):
        return {"runtime": {"task_queue_size": 2, "workers": 2}}


class TestNestedConcurrency5W256Q(TestNestedConcurrency1W2Q):
    @pytest.fixture(scope="class")
    def test_config(self):
        return {"runtime": {"task_queue_size": 256, "workers": 5}}
