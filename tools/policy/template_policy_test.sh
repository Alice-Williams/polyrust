#!/usr/bin/env bash
set -euo pipefail

readonly runfiles="${RUNFILES_DIR:-$0.runfiles}"
readonly root="${runfiles}/${TEST_WORKSPACE}"

mapfile -d '' java_renderers < <(
  find -L "${root}/crates/backend-java/src/render" -type f -name '*.rs' -print0
)
if (( ${#java_renderers[@]} == 0 )); then
  echo "Java renderer child modules are missing from policy runfiles" >&2
  exit 1
fi

python3 \
  "${root}/tools/policy/template_policy.py" \
  verify \
  "${root}/crates/codegen/src/rendering.rs" \
  "${root}/crates/backend-java/src/render.rs" \
  "${java_renderers[@]}"
