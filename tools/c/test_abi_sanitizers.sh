#!/usr/bin/env bash
set -euo pipefail

runfiles="${RUNFILES_DIR:-$0.runfiles}"
generated="$(find "$runfiles" -path '*/abi-generated/src/generated.c' -print -quit)"
test_source="$(find "$runfiles" -path '*/backend-c/test/abi_shapes_test.c' -print -quit)"
test -n "$generated"
test -n "$test_source"
test "$(gcc-14 -dumpfullversion)" = "14.2.0"
test "$(dpkg-query -W -f='${Version}' gcc-14)" = "14.2.0-19"
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
  "$test_source"
)
gcc-14 "${common[@]}" -fsanitize=address -o "$work/asan-test"
ASAN_OPTIONS=detect_leaks=1:halt_on_error=1 "$work/asan-test"
gcc-14 "${common[@]}" -fsanitize=undefined -fno-sanitize-recover=undefined -o "$work/ubsan-test"
UBSAN_OPTIONS=halt_on_error=1 "$work/ubsan-test"
