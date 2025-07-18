"""
Test scenario runner for component integration tests.
"""

from pathlib import Path
import pytest
from pytest import FixtureRequest
from testing_utils import Scenario, LogContainer


class CitScenario(Scenario):
    """
    CIT test scenario definition.
    """

    @pytest.fixture(scope="class")
    def logs_bin(self, bin_path: Path, logs: LogContainer) -> LogContainer:
        """
        Logs with messages generated strictly by the tested code.

        Parameters
        ----------
        bin_path : Path
            Path to test scenario executable.
        logs : LogContainer
            Unfiltered logs.
        """
        return logs.get_logs_by_field(field="target", pattern=f"{bin_path.name}.*")

    @pytest.fixture(scope="class")
    def logs_info_level(self, logs_bin: LogContainer) -> LogContainer:
        """
        Logs with messages with INFO level.

        Parameters
        ----------
        logs_bin : LogContainer
            Logs with messages generated strictly by the tested code.
        """
        return logs_bin.get_logs_by_field(field="level", value="INFO")

    @pytest.fixture(scope="function", autouse=True)
    def print_to_report(
        self, request: FixtureRequest, logs: LogContainer, logs_bin: LogContainer
    ) -> None:
        """
        Print traces to stdout.
        - if "--test-traces" is set - only executable-specific tests are printed
        - else - all logs are printed

        Parameters
        ----------
        request : FixtureRequest
            Test request built-in fixture.
        logs : LogContainer
            Test scenario execution logs.
        logs_bin : LogContainer
            Logs with messages generated strictly by the tested code.
        """
        if request.config.getoption("--test-traces"):
            traces = logs_bin
        else:
            traces = logs

        for trace in traces:
            print(trace)
