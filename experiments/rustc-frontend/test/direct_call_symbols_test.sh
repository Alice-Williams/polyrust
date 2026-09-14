#!/usr/bin/env bash
set -euo pipefail
source="$1"
zig="$2"
work="${TEST_TMPDIR:?}/direct-call-symbols"
mkdir -p "$work"
for compiler in gcc zig; do
    command=(gcc-14)
    if [ "$compiler" = zig ]; then command=("$zig"); fi
    for optimization in 0 2; do
        object="$work/$compiler-o$optimization.o"
        "${command[@]}" -std=c17 -Wall -Wextra -Wpedantic -Werror \
            -Wstrict-prototypes -Wmissing-prototypes "-O$optimization" -c "$source" -o "$object"
        nm --defined-only "$object" > "$object.symbols"
        awk '$2 == "T" { print $3 }' "$object.symbols" > "$object.public"
        test "$(< "$object.public")" = poly_score
        if [ "$optimization" = 0 ]; then
            test "$(awk '$2 == "t" { count++ } END { print count+0 }' "$object.symbols")" = 5
        fi
    done
done
echo 'Only Rust-public score is exported; all five private helpers have local O0 symbols'
