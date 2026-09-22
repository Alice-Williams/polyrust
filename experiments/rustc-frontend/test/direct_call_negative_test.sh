#!/usr/bin/env bash
# Valid Rust outside the registered call subset must never create/replace C.
set -euo pipefail
adapter="$1"
work="${TEST_TMPDIR:?}/direct-call-negative"
mkdir -p "$work"

reject() {
    local name="$1" reason="$2" source="$3"
    printf '%s\n' "$source" > "$work/$name.rs"
    for existing in no yes; do
        local output="$work/$name-$existing.c"
        if [ "$existing" = yes ]; then printf 'preserved\n' > "$output"; fi
        if "$adapter" "$work/$name.rs" "$output" > "$work/$name.log" 2>&1; then
            echo "incorrectly admitted $name" >&2
            exit 1
        fi
        # This callback-only diagnostic also proves rustc analysis succeeded.
        grep -Fq 'unsupported Rust:' "$work/$name.log"
        grep -Fq "$reason" "$work/$name.log"
        if [ "$existing" = yes ]; then
            test "$(< "$output")" = preserved
        else
            test ! -e "$output"
        fi
    done
}

reject indirect 'resolved ordinary functions' '
fn helper(value: i32) -> i32 { value }
pub fn score(value: i32) -> i32 { let call: fn(i32) -> i32 = helper; call(value) }'
reject generic 'generic or mismatched' '
fn helper<T>(value: T) -> T { value }
pub fn score(value: i32) -> i32 { helper(value) }'
reject foreign 'foreign direct calls' '
pub fn score(value: i32) -> i32 { let ignored = std::process::id(); value }'
reject method 'expression mapping is not implemented' '
struct Value { input: i32 }
impl Value { fn get(&self) -> i32 { self.input } }
pub fn score(input: i32) -> i32 { let value = Value { input }; value.get() }'
reject pointer_signature 'signatures require admitted scalar source types' '
fn helper(value: &i32) -> i32 { *value }
pub fn score(value: i32) -> i32 { helper(&value) }'
reject foreign_abi 'ordinary Rust signatures' '
extern "C" fn helper(value: i32) -> i32 { value }
pub fn score(value: i32) -> i32 { helper(value) }'
reject private_entry 'entry must be externally reachable' '
fn score(value: i32) -> i32 { value }'
reject global_read 'only resolved local value paths' '
static VALUE: i32 = 1;
fn helper() -> i32 { VALUE }
pub fn score(value: i32) -> i32 { let ignored = value; helper() }'
reject recursive 'C typed lowering:' '
fn helper(value: i32) -> i32 { helper(value) }
pub fn score(value: i32) -> i32 { helper(value) }'
reject mutual 'C typed lowering:' '
fn first(value: i32) -> i32 { second(value) }
fn second(value: i32) -> i32 { first(value) }
pub fn score(value: i32) -> i32 { first(value) }'

printf 'pub fn score(value: i32) -> i32 { helper(value) }\nfn helper(value: i32) -> i32 { value }\n' > "$work/control.rs"
"$adapter" "$work/control.rs" "$work/control.c"
test -s "$work/control.c"
echo 'Direct-call unsupported shapes fail after Rust analysis and preserve output'
