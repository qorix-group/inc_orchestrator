#!/usr/bin/env bash

# Exit immediately if any command fails
set -e

echo "Does not work yet"
if [ -f "virtualization/_BUILD" ]; then
  echo "Error: _BUILD file found in src/virtualization/. Please rename it to BUILD. If You do so, //.... target will stop working until this is not reverted. Sorry for the inconvenience."
  exit 1
fi

  if [[ ! -f "$HOME/.netrc" ]]; then
    echo "Error: Either SCORE_QNX_USER/SCORE_QNX_PASSWORD must be set, or $HOME/.netrc must exist."
# # Check if SCORE_QNX_USER is set
# if [[ -z "${SCORE_QNX_USER}" ]]; then
#   echo "Error: SCORE_QNX_USER is not set. This shall be myQNX portal username."
#   exit 1
# fi

# # Check if SCORE_QNX_PASSWORD is set
# if [[ -z "${SCORE_QNX_PASSWORD}" ]]; then
#   echo "Error: SCORE_QNX_PASSWORD is not set. This shall be myQNX portal password."
#   exit 1
# fi

# echo "Note: If it fails with 'Error downloading [https://www.qnx.com/download/download/79858/installation.tgz' access this link from webbrowser and accept the license agreement."

# bazel run --sandbox_debug --verbose_failures --platforms=@score_toolchains_rust//platforms:aarch64-unknown-qnx8_0 --credential_helper=*.qnx.com=%workspace%/scripts/internal/qnx_creds.py //virtualization:run_qemu
