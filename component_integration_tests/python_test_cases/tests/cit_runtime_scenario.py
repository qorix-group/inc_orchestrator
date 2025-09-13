"""
Runtime test scenario runner for component integration tests.
"""

import time
from queue import Empty, Queue
from socket import AF_INET, SOCK_STREAM, socket
from subprocess import PIPE, Popen
from threading import Thread
from typing import Generator

import pytest
from testing_utils import BuildTools, CargoTools, LogContainer, Scenario


# TODO: Should be moved to testing_utils
class NetHelper:
    @staticmethod
    def connection_builder(ip: str, port: int, timeout: float | None = 3.0) -> socket:
        """
        Create and return a socket connected to the server.

        Parameters
        ----------
        ip : str
            IP address of the server.
        port : int
            Port number of the server.
        timeout : float | None
            Connection timeout in seconds. 0 for non-blocking mode, None for blocking mode.
        """
        s = socket(AF_INET, SOCK_STREAM)
        s.settimeout(timeout)
        s.connect((ip, port))
        return s


# TODO: Should support ResultEntry and LogContainer classes
class Executable:
    def __init__(self, command: list[str]) -> None:
        self._proc: Popen[str] = Popen(
            command, stdout=PIPE, stderr=PIPE, text=True, bufsize=1
        )
        self._stdout: list[str] = []
        self._queue: Queue[str] = Queue()
        self._stdout_reader: Thread = Thread(
            target=Executable.process_output, args=(self._proc.stdout, self._queue)
        )
        self._stdout_reader.start()

    def __enter__(self) -> "Executable":
        return self

    def __exit__(self, _type, _value, _traceback) -> None:
        self.terminate()

    @staticmethod
    def process_output(stream, queue):
        """
        Read output line by line and put it to the queue.
        Reading stream of the running process can block, so it should be done in a separate thread.
        """
        for line in iter(stream.readline, ""):
            queue.put(line)

    def terminate(self) -> None:
        """
        Terminate the process and update stdout.
        """
        self._proc.terminate()
        self._update_stdout()
        self._stdout_reader.join()

    def _read_stdout_line(self, timeout) -> str:
        """
        Read a line from stdout queue with timeout.

        Parameters
        ----------
        timeout : float
            Timeout in seconds.
        """
        try:
            line = self._queue.get(block=True, timeout=timeout).strip()
            self._stdout.append(line)
            return line
        except Empty:
            return None

    def _update_stdout(self) -> list[str]:
        """
        Update stdout with all available lines.
        """
        new_data = []
        while not self._queue.empty():
            new_data.append(self._read_stdout_line(timeout=0))
        return new_data

    def get_stdout_until_now(self) -> list[str]:
        """
        Get all stdout lines until now.
        """
        self._update_stdout()
        return self._stdout[:]

    # TODO: Should be removed when LogContainer is supported
    def get_stdout_logcontainer(self) -> LogContainer:
        """
        Get all stdout lines until now as LogContainer.
        """
        self._update_stdout()
        return LogContainer(self.get_stdout_until_now())

    def wait_for_log(self, pattern: str, timeout: float = 5.0):
        """
        Wait for a specific log substring in stdout until timeout.

        Parameters
        ----------
        pattern : str
            Substring to wait for.
        timeout : float
            Timeout in seconds.
        """
        start = time.time()
        stdout = self.get_stdout_until_now()
        for line in stdout:
            if pattern in line:
                return

        now = time.time()
        while now - start < timeout:
            to_timeout = timeout - (now - start)
            stdout_line = self._read_stdout_line(to_timeout)
            if stdout_line is None:
                break

            if pattern in stdout_line:
                return
            now = time.time()

        raise TimeoutError(f'Timeout waiting for "{pattern}" in stdout')


class CitRuntimeScenario(Scenario):
    """
    CIT test scenario definition for interactive testing with binary running continuously.
    It requires executable to be running for the duration of the tests.
    """

    @pytest.fixture(scope="class")
    def build_tools(self, *args, **kwargs) -> BuildTools:
        """
        Build tools used to handle test scenario.
        """
        return CargoTools()

    @pytest.fixture(scope="class")
    def results(
        self,
        process,
        execution_timeout: float,
        *args,
        **kwargs,
    ):
        raise NotImplementedError("Not used in runtime scenarios.")

    @pytest.fixture(scope="class")
    def logs(self, results, *args, **kwargs):
        raise NotImplementedError("Not used in runtime scenarios.")

    @pytest.fixture(scope="function")
    def executable(
        self, command: list[str], *args, **kwargs
    ) -> Generator[Executable, None, None]:
        """
        Start the executable process and terminate it after tests.
        """
        with Executable(command) as exec:  # TODO: Pass LogContainer as a parameter?
            yield exec

    @pytest.fixture(autouse=True)
    def print_to_report(
        self,
        request: pytest.FixtureRequest,
        executable: Executable,
    ) -> Generator[None, None, None]:
        """
        Print traces to stdout.

        Allowed "--traces" values:
        - "none" - show no traces.
        - "target" - show traces generated by test code.
        - "all" - show all traces.

        Parameters
        ----------
        request : FixtureRequest
            Test request built-in fixture.
        executable : Executable
            Executable instance.
        """
        traces_param = request.config.getoption("--traces")
        if traces_param not in ("none", "target", "all"):
            raise RuntimeError(f'Invalid "--traces" value: {traces_param}')

        yield  # Traces shoud be printed after test execution

        match traces_param:
            case "all":
                traces = executable.get_stdout_until_now()
            case "target":
                raise NotImplementedError("Not implemented")
            case "none":
                traces = []
            case _:
                raise RuntimeError(f'Invalid "--traces" value: {traces_param}')

        if traces:
            print("\n")
            for trace in traces:
                print(trace)
