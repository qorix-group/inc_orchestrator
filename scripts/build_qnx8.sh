#!/usr/bin/env bash

echo "Pass the BAZEL targets as arguments, e.g.: //src/... "

# Exit immediately if any command fails
set -e

# Check if SCORE_QNX_USER is set
if [[ -z "${SCORE_QNX_USER}" ]]; then
  echo "Error: SCORE_QNX_USER is not set. This shall be myQNX portal username."
  exit 1
fi

# Check if SCORE_QNX_PASSWORD is set
if [[ -z "${SCORE_QNX_PASSWORD}" ]]; then
  echo "Error: SCORE_QNX_PASSWORD is not set. This shall be myQNX portal password."
  exit 1
fi

echo "Note: If it fails with 'Error downloading [https://www.qnx.com/download/download/79858/installation.tgz' access this link from webbrowser and accept the license agreement."

# Use default argument //src/... if none provided
if [ $# -eq 0 ]; then
  set -- //src/...
fi

bazel build --sandbox_debug --verbose_failures --platforms=@score_toolchains_rust//platforms:aarch64-unknown-qnx8_0 --credential_helper=*.qnx.com=%workspace%/scripts/internal/qnx_creds.py "$@"
