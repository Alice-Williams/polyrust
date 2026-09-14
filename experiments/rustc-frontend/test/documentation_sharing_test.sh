#!/usr/bin/env bash
set -euo pipefail
readonly probe="$1"
readonly work="${TEST_TMPDIR:?}/shared-documentation"
mkdir -p "$work"
head -c 1048576 /dev/zero | tr '\0' x > "$work/module.md"
{
    printf '%s\n' '#![doc = include_str!("module.md")]' 'pub struct Wide {'
    for ((field=0; field<256; field++)); do
        printf '    pub field_%s: i32,\n' "$field"
    done
    printf '%s\n' '}' 'pub fn score(input: i32) -> i32 {' '    let wide = Wide {'
    for ((field=0; field<256; field++)); do
        printf '        field_%s: input,\n' "$field"
    done
    printf '%s\n' '    };' '    wide.field_0' '}'
} > "$work/input.rs"
"$probe" "$work/input.rs" --input "$work/module.md"
