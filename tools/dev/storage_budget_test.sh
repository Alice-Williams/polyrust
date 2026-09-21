#!/usr/bin/env bash
set -euo pipefail
readonly root="${RUNFILES_DIR:-$0.runfiles}/${TEST_WORKSPACE}"
exec python3 "${root}/tools/dev/storage_budget_test.py"
