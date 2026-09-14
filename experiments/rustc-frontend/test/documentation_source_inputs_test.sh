#!/usr/bin/env bash
set -euo pipefail
readonly adapter="$1"
readonly consumer="$2"
readonly work="${TEST_TMPDIR:?}/documentation-source-inputs"
mkdir -p "$work"
test "$(gcc-14 -dumpfullversion)" = "14.2.0"
for form in include module; do
    if test "$form" = include; then
        printf '%s\n' '"DOC_SOURCE_INPUT"' > "$work/piece.rs"
        printf '%s\n' '#[doc = include!("piece.rs")]' \
            'pub fn score(input: i32) -> i32 { input }' > "$work/input.rs"
    else
        printf '%s\n' '//! DOC_SOURCE_INPUT' \
            'pub struct Record { pub value: i32 }' > "$work/piece.rs"
        printf '%s\n' 'mod piece;' 'pub fn score(input: i32) -> i32 {' \
            'let record = piece::Record { value: input }; record.value }' > "$work/input.rs"
    fi
    for state in absent existing; do
        output="$work/$form-$state.c"
        if test "$state" = existing; then
            printf 'preserve existing artifact\n' > "$output"
            cp "$output" "$work/expected.c"
        fi
        if "$adapter" "$work/input.rs" "$output" > "$work/rejected.log" 2>&1; then
            echo "undeclared $form source input was admitted" >&2
            exit 1
        fi
        grep -q 'undeclared compiler file input:' "$work/rejected.log"
        grep -q 'piece.rs' "$work/rejected.log"
        if test "$state" = absent; then test ! -e "$output"; else cmp "$output" "$work/expected.c"; fi
    done
    output="$work/$form-declared.c"
    "$adapter" "$work/input.rs" "$output" --input "$work/piece.rs"
    test "$(grep -c DOC_SOURCE_INPUT "$output")" = 1
    for optimization in 0 2; do
        gcc-14 -std=c17 -Wall -Wextra -Wpedantic -Werror -Wstrict-prototypes \
            -Wmissing-prototypes "-O$optimization" "$output" "$consumer" -o "$work/native"
        printf '%s\n' -2147483648 -1 0 1 2147483647 > "$work/values"
        "$work/native" < "$work/values" > "$work/actual"
        cmp "$work/values" "$work/actual"
    done
done
