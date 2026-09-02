#!/usr/bin/env bash
set -euo pipefail

readonly runfiles="${RUNFILES_DIR:-$0.runfiles}"
readonly root="${runfiles}/${TEST_WORKSPACE}"
readonly mode="${1:-verify}"

if [[ "${mode}" == "self-test" ]]; then
    python3 "${root}/tools/source-policy/source_policy.py" self-test
else
    python3 "${root}/tools/source-policy/source_policy.py" verify "${root}"
fi
