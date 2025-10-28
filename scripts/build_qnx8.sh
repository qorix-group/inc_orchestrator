#!/usr/bin/env bash

echo "Pass the BAZEL targets as arguments, e.g.: //src/... "

# Exit immediately if any command fails
set -e


# Check if SCORE_QNX_USER and SCORE_QNX_PASSWORD are set
if [[ -z "${SCORE_QNX_USER}" || -z "${SCORE_QNX_PASSWORD}" ]]; then
  # If either is missing, check for .netrc
  if [[ ! -f "$HOME/.netrc" ]]; then
    echo "Error: Either SCORE_QNX_USER/SCORE_QNX_PASSWORD must be set, or $HOME/.netrc must exist."
    exit 1
  fi
fi

echo "Note: If it fails with 'Error downloading [https://www.qnx.com/download/download/79858/installation.tgz' access this link from webbrowser and accept the license agreement."

# Use default argument //src/... if none provided
if [ $# -eq 0 ]; then
  set -- //src/...
fi

bazel build  --platforms=@score_toolchains_rust//platforms:aarch64-unknown-qnx8_0 --credential_helper=*.qnx.com=%workspace%/scripts/internal/qnx_creds.py "$@"
