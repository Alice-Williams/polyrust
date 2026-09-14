#!/usr/bin/env bash
set -euo pipefail
adapter="$1"
root="$2"
module="$3"
work="${TEST_TMPDIR:?}/export-negative"
mkdir -p "$work"

reject() {
    local case="$1" code="$2" source="$3"
    printf '%s\n' "$source" > "$work/$case.rs"
    for existing in no yes; do
        local output="$work/$case-$existing.c"
        if [ "$existing" = yes ]; then printf 'preserved\n' > "$output"; fi
        if "$adapter" "$work/$case.rs" "$output" > "$work/$case.log" 2>&1; then
            echo "incorrectly admitted $case" >&2
            exit 1
        fi
        grep -Fq "$code" "$work/$case.log"
        if [ "$existing" = yes ]; then
            test "$(< "$output")" = preserved
        else
            test ! -e "$output"
        fi
    done
}

reject private_function E0603 '
mod hidden { fn helper(input: i32) -> i32 { input } }
pub fn score(input: i32) -> i32 { hidden::helper(input) }'
reject private_field E0616 '
mod hidden { pub struct Value { secret: i32 } }
pub fn score(input: i32) -> i32 { let value = hidden::Value { secret: input }; value.secret }'
reject restricted_reexport E0365 '
mod hidden { pub(crate) struct Value { pub value: i32 } }
pub use hidden::Value;
pub fn score(input: i32) -> i32 { input }'
reject sibling_scope E0603 '
mod first { pub(in crate::first) fn helper(input: i32) -> i32 { input } }
mod second { pub fn score(input: i32) -> i32 { crate::first::helper(input) } }'

# A readable public path-selected module is still an explicit compiler input.
for existing in no yes; do
    output="$work/undeclared-$existing.c"
    if [ "$existing" = yes ]; then printf 'preserved\n' > "$output"; fi
    if "$adapter" "$root" "$output" > "$work/undeclared.log" 2>&1; then exit 1; fi
    grep -Fq 'undeclared compiler file input' "$work/undeclared.log"
    if [ "$existing" = yes ]; then test "$(< "$output")" = preserved; else test ! -e "$output"; fi
done
"$adapter" "$root" "$work/declared.c" --input "$module"
test -s "$work/declared.c"
echo 'Rust privacy and declared public-module input rejection preserve output'
