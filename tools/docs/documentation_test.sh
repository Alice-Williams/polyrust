#!/usr/bin/env bash
set -euo pipefail

readonly runfiles="${RUNFILES_DIR:-$0.runfiles}"
readonly root="${runfiles}/${TEST_WORKSPACE}"
python3 "${root}/tools/docs/check_links.py" "${root}"
