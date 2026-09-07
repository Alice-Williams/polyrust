#!/usr/bin/env bash
set -euo pipefail

readonly runfiles="${RUNFILES_DIR:-$0.runfiles}"
python3 "${runfiles}/${TEST_WORKSPACE}/tools/policy/source_discovery_test.py"
