#!/usr/bin/env bash
set -euo pipefail

runfiles="${RUNFILES_DIR:-$0.runfiles}"
generated="$(find "$runfiles" -path '*/generated/c/src/generated.c' -print -quit)"
if test -z "$generated"; then
  generated="$(find "$runfiles" -path '*/test-generated/src/generated.c' -print -quit)"
fi
test -n "$generated"
test "$(gcc-14 -dumpfullversion)" = "14.2.0"
package="$(dirname "$(dirname "$generated")")"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
cp -RL "$package/." "$work/"

common=(
  -std=c17
  -Wall
  -Wextra
  -Wpedantic
  -Werror
  -I "$work/src"
  "$work/src/runtime.c"
  "$work/src/generated.c"
  "$work/tests/generated_test.c"
  -lm
)
gcc-14 "${common[@]}" -fsanitize=address -o "$work/asan-test"
ASAN_OPTIONS=detect_leaks=1:halt_on_error=1 "$work/asan-test"
gcc-14 "${common[@]}" -fsanitize=undefined -fno-sanitize-recover=undefined -o "$work/ubsan-test"
UBSAN_OPTIONS=halt_on_error=1 "$work/ubsan-test"
