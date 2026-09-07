#!/usr/bin/env bash
set -euo pipefail
readonly root="${RUNFILES_DIR:-$0.runfiles}/${TEST_WORKSPACE}"
python3 "${root}/tools/ci/writable_copy_test.py" "${root}"
