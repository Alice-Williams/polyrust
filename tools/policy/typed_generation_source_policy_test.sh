#!/usr/bin/env bash
set -euo pipefail

readonly runfiles="${RUNFILES_DIR:-$0.runfiles}"
readonly root="${runfiles}/${TEST_WORKSPACE}"

readonly source_list="${TEST_TMPDIR}/java-production-sources.list"
if ! find -L "${root}/crates/backend-java/src" -type f -name '*.rs' \
  ! -path '*/src/tests/*' -print0 > "${source_list}"; then
  echo "Java production source discovery failed" >&2
  exit 1
fi
mapfile -d '' java_sources < "${source_list}"
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
