#!/usr/bin/env bash
set -euo pipefail

consumer="$1"
runfiles="${RUNFILES_DIR:-$0.runfiles}"
generated="$(find "$runfiles" -path '*/generated/c/src/generated.c' -print -quit)"
test -n "$generated"
test -f "$consumer"
test "$(gcc-14 -dumpfullversion)" = "14.2.0"
package="$(dirname "$(dirname "$generated")")"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

common=(
  -std=c17
  -Wall
  -Wextra
  -Wpedantic
  -Werror
  -I "$package/src"
  "$package/src/runtime.c"
  "$package/src/generated.c"
  "$consumer"
  -lm
)
gcc-14 "${common[@]}" -fsanitize=address -o "$work/asan-test"
ASAN_OPTIONS=detect_leaks=1:halt_on_error=1 "$work/asan-test"
gcc-14 "${common[@]}" -fsanitize=undefined -fno-sanitize-recover=undefined -o "$work/ubsan-test"
UBSAN_OPTIONS=halt_on_error=1 "$work/ubsan-test"
