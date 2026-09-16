#!/usr/bin/env bash
set -euo pipefail
readonly root="${RUNFILES_DIR:-$0.runfiles}/${TEST_WORKSPACE}"
python3 "${root}/tools/ci/rust_compatibility_policy_test.py" \
  "${root}/.github/workflows/ci.yml" \
  "${root}/tools/release/release_gate.sh"
