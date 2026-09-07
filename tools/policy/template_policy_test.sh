#!/usr/bin/env bash
set -euo pipefail

readonly runfiles="${RUNFILES_DIR:-$0.runfiles}"
readonly root="${runfiles}/${TEST_WORKSPACE}"

readonly source_list="${TEST_TMPDIR}/java-renderer-sources.list"
if ! find -L "${root}/crates/backend-java/src/render" -type f -name '*.rs' \
  -print0 > "${source_list}"; then
  echo "Java renderer source discovery failed" >&2
  exit 1
fi
mapfile -d '' java_renderers < "${source_list}"
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
