#!/usr/bin/env bash
set -euo pipefail

runfiles="${RUNFILES_DIR:-$0.runfiles}"
generated="$(find "$runfiles" -path '*/generated/cpp/src/generated.cc' -print -quit)"
if test -z "$generated"; then
  generated="$(find "$runfiles" -path '*/test-generated/src/generated.cc' -print -quit)"
fi
test -n "$generated"
package="$(dirname "$(dirname "$generated")")"
mapfile -t sources < <(find -L "$package" -type f \( -name '*.cc' -o -name '*.hpp' \) -print | sort)
test "${#sources[@]}" -ge 5
if grep -n $'\t\|\r\|[[:blank:]]$' "${sources[@]}"; then
  echo "C++ style gate found tabs, carriage returns, or trailing whitespace" >&2
  exit 1
fi
grep -q '^#pragma once$' "$package/src/generated.hpp"
grep -q '^#pragma once$' "$package/src/runtime.hpp"
grep -q '^namespace polyrust_generated {' "$package/src/generated.hpp"
