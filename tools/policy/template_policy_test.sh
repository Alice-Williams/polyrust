#!/usr/bin/env bash
set -euo pipefail

readonly runfiles="${RUNFILES_DIR:-$0.runfiles}"
readonly root="${runfiles}/${TEST_WORKSPACE}"

python3 \
  "${root}/tools/policy/template_policy.py" \
  verify \
  "${root}/crates/codegen/src/rendering.rs" \
  "${root}/crates/backend-java/src/render.rs"
