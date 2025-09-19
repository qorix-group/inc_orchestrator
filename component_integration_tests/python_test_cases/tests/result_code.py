class ResultCode:
    """
    Test scenario exit codes.
    """

    SUCCESS = 0
    PANIC = 101
    SIGABRT = -6
    SIGKILL = -9
    SIGSEGV = -11
    SIGTERM = -15
