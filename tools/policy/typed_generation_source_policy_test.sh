#!/usr/bin/env bash
set -euo pipefail

readonly runfiles="${RUNFILES_DIR:-$0.runfiles}"
readonly root="${runfiles}/${TEST_WORKSPACE}"

mapfile -d '' java_sources < <(
  find -L "${root}/crates/backend-java/src" -type f -name '*.rs' \
    ! -path '*/src/tests/*' -print0
)
if (( ${#java_sources[@]} == 0 )); then
  echo "Java production sources are missing from policy runfiles" >&2
  exit 1
fi

python3 \
  "${root}/tools/policy/typed_generation_source_policy.py" \
  verify \
  "${root}/crates/build/src/typed_program.rs" \
  "${root}/crates/build/src/capabilities/"*.rs \
  "${root}/crates/codegen/src/linking.rs" \
  "${root}/crates/codegen/src/target_ast.rs" \
  "${java_sources[@]}"
