#!/bin/bash

# For the documentation, please refer to component_integration_tests/python_test_cases/run_tests.py
#
# Example usage:
#     Run all tests:
#     ./run_component_tests.sh
#
#     Run all tests in a specific file (file path must be relative to python_test_cases folder):
#     ./run_component_tests.sh tests/runtime/worker/test_worker_basic.py
#
#     Run a specific scenario:
#     ./run_component_tests.sh tests/runtime/worker/test_worker_basic.py::TestRuntimeOneWorkerOneTask

FILE_PATH=$(realpath "$0")
FILE_DIR=$(dirname "$FILE_PATH")

cd "$FILE_DIR"/../component_integration_tests/python_test_cases || exit 1
./run_tests.py "$@"
