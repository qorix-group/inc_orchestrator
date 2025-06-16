import pytest
from testing_tools.log_container import LogContainer


class TestSingleSequence1W2Q:
    @pytest.fixture(scope="class")
    def test_case_name(self):
        return "orchestration.single_sequence"

    @pytest.fixture(scope="class")
    def test_config(self):
        return {"runtime": {"task_queue_size": 2, "workers": 1}}

    def test_execution_order_one_branch(self, test_results: LogContainer):
        action1 = test_results.find_log(field="message", pattern="Action1 was executed")
        action2 = test_results.find_log(field="message", pattern="Action2 was executed")
        action3 = test_results.find_log(field="message", pattern="Action3 was executed")
        # Assert that execution_order is chronological by timestamp
        assert action1.timestamp < action2.timestamp < action3.timestamp, (
            "Actions were not executed in the expected order: Action1, Action2, Action3"
        )


class TestSingleSequence2W2Q(TestSingleSequence1W2Q):
    @pytest.fixture(scope="class")
    def test_config(self):
        return {"runtime": {"task_queue_size": 2, "workers": 2}}


class TestNestedSequence1W2Q:
    @pytest.fixture(scope="class")
    def test_case_name(self):
        return "orchestration.nested_sequence"

    @pytest.fixture(scope="class")
    def test_config(self):
        return {"runtime": {"task_queue_size": 2, "workers": 1}}

    def test_outer_sequence_executed(self, test_results: LogContainer):
        assert test_results.contains_log(field="message", pattern="OuterAction*"), (
            "OuterAction was not executed as expected"
        )

    def test_inner_sequence_executed(self, test_results: LogContainer):
        assert test_results.contains_log(field="message", pattern="InnerAction*"), (
            "InnerAction was not executed as expected"
        )

    def test_execution_order_sequence_in_sequence(self, test_results: LogContainer):
        expected_order = [
            "OuterAction1 was executed",
            "InnerAction1 was executed",
            "InnerAction2 was executed",
            "OuterAction2 was executed",
        ]
        execution_order = [
            log.message
            for log in test_results.get_logs_by_field(
                field="message", pattern="was executed"
            )
        ]
        assert execution_order == expected_order, (
            "Actions were not executed in the expected order"
        )


class TestNestedSequence2W2Q(TestNestedSequence1W2Q):
    @pytest.fixture(scope="class")
    def test_config(self):
        return {"runtime": {"task_queue_size": 256, "workers": 2}}
