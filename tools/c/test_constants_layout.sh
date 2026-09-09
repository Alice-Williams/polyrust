#!/usr/bin/env bash
set -euo pipefail

probe="$1"
test -f "$probe"
test "$(gcc-14 -dumpfullversion)" = "14.2.0"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
common=(
  -std=c17 -Wall -Wextra -Wpedantic -Werror
  -Wstrict-prototypes -Wmissing-prototypes
  -fno-fast-math -ffp-contract=off -fsigned-char -fno-short-enums
)
for optimization in -O0 -O2; do
  gcc-14 "${common[@]}" "$optimization" "$probe" -o "$work/probe"
  "$work/probe"
done
if gcc-14 "${common[@]}" -DPOLY_UNSAFE_ARITHMETIC "$probe" -o "$work/invalid" 2>"$work/negative.log"; then
  echo "C constant negative control unexpectedly accepted signed overflow" >&2
  exit 1
fi
grep -F -q 'poly_signed_overflow' "$work/negative.log"
