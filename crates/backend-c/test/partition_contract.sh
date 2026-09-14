#!/usr/bin/env bash
set -euo pipefail
binary="$1"
shift
directory="${TEST_TMPDIR:?}/partition-contract"
mkdir -p "$directory"
"$binary" --list --format terse | LC_ALL=C sort > "$directory/all"
test -s "$directory/all"
# Prove the pinned libtest applies --exact to --skip too: no test is named just
# this module prefix, so exact exclusion must retain the complete inventory.
"$binary" --list --format terse --exact --skip 'dialect::shared::' |
    LC_ALL=C sort > "$directory/exact-prefix"
cmp "$directory/all" "$directory/exact-prefix"
skips=()
for case in "$@"; do
    skips+=(--skip "$case")
    inventory=$("$binary" --list --format terse --exact "$case")
    test "$inventory" = "$case: test"
    printf '%s\n' "$inventory"
done > "$directory/expensive"
"$binary" --list --format terse --exact "${skips[@]}" > "$directory/ordinary"
# Do not uniq: overlapping partitions must fail as well as missing tests.
LC_ALL=C sort "$directory/ordinary" "$directory/expensive" > "$directory/combined"
cmp "$directory/all" "$directory/combined"
