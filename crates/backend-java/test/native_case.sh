#!/usr/bin/env bash
set -euo pipefail
mode="$1"
binary="$2"
shift 2
if [[ "$mode" == run ]]; then
    case="$1"
    shift
    inventory=$("$binary" --list --exact "$case" --format terse)
    test "$inventory" = "$case: test"
    exec "$binary" --exact "$case" --nocapture "$@"
fi
test "$mode" = contract
directory="${TEST_TMPDIR:?}/partition-contract"
mkdir -p "$directory"
"$binary" --list --format terse | LC_ALL=C sort > "$directory/all"
test -s "$directory/all"
skips=()
for case in "$@"; do
    skips+=(--skip "$case")
    inventory=$("$binary" --list --exact "$case" --format terse)
    test "$inventory" = "$case: test"
    printf '%s\n' "$inventory"
done > "$directory/native"
"$binary" --list --format terse "${skips[@]}" > "$directory/ordinary"
LC_ALL=C sort "$directory/ordinary" "$directory/native" > "$directory/combined"
cmp "$directory/all" "$directory/combined"
