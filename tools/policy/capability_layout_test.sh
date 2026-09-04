#!/usr/bin/env bash
set -euo pipefail

readonly runfiles="${RUNFILES_DIR:-$0.runfiles}"
readonly root="${runfiles}/${TEST_WORKSPACE}"

python3 \
  "${root}/tools/policy/capability_layout_test.py" \
  "${root}/crates/build/src/capabilities" \
  "${root}/crates/backend-java/src/capabilities"
