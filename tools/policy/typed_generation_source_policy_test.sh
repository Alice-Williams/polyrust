#!/usr/bin/env bash
set -euo pipefail

readonly runfiles="${RUNFILES_DIR:-$0.runfiles}"
readonly root="${runfiles}/${TEST_WORKSPACE}"

python3 \
  "${root}/tools/policy/typed_generation_source_policy.py" \
  verify \
  "${root}/crates/build/src/typed_program.rs" \
  "${root}/crates/build/src/capabilities/"*.rs \
  "${root}/crates/codegen/src/linking.rs" \
  "${root}/crates/codegen/src/target_ast.rs" \
  "${root}/crates/backend-java/src/"*.rs
