#!/usr/bin/env bash
set -euo pipefail

runfiles="${RUNFILES_DIR:-$0.runfiles}"
generated="$(find "$runfiles" -path '*/generated/cpp/src/generated.cc' -print -quit)"
if test -z "$generated"; then
  generated="$(find "$runfiles" -path '*/test-generated/src/generated.cc' -print -quit)"
fi
test -n "$generated"
test "$(g++ -dumpfullversion)" = "14.2.0"
test "$(dpkg-query -W -f='${Version}' g++-14)" = "14.2.0-19"
package="$(dirname "$(dirname "$generated")")"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
cp -RL "$package/." "$work/"

common=(
  -std=c++20
  -Wall
  -Wextra
  -Wpedantic
  -Werror
  -Wno-dangling-reference
  -Wno-pedantic
  -I "$work/src"
  "$work/src/generated.cc"
  "$work/tests/generated_test.cc"
)
g++ "${common[@]}" -fsanitize=address -o "$work/asan-test"
ASAN_OPTIONS=detect_leaks=1:halt_on_error=1 "$work/asan-test"
g++ "${common[@]}" -fsanitize=undefined -fno-sanitize-recover=undefined -o "$work/ubsan-test"
UBSAN_OPTIONS=halt_on_error=1 "$work/ubsan-test"
