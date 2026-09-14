#!/usr/bin/env bash
set -euo pipefail
readonly adapter="$1"
readonly fixtures="$(dirname "$2")"
readonly work="${TEST_TMPDIR}/compiler-boundary"
mkdir -p "$work"
for case in foreign_abi unstable packed aligned alias_signature alias_field alias_local alias_external alias_constructor; do
    if "$adapter" "$fixtures/$case.rs" "$work/$case.c" > "$work/$case.log" 2>&1; then
        echo "incorrectly admitted $case" >&2
        exit 1
    fi
    test ! -e "$work/$case.c"
    cp "$fixtures/inputs.txt" "$work/$case.c"
    if "$adapter" "$fixtures/$case.rs" "$work/$case.c" > "$work/$case-again.log" 2>&1; then
        exit 1
    fi
    cmp "$fixtures/inputs.txt" "$work/$case.c"
done
grep -q 'unsupported Rust: entry must use a safe non-variadic ordinary Rust signature' "$work/foreign_abi.log"
grep -q 'E0554' "$work/unstable.log"
for case in packed aligned; do
    grep -q 'unsupported Rust: custom Rust record representations are not implemented' "$work/$case.log"
done
for case in alias_signature alias_field alias_local alias_external alias_constructor; do
    grep -q 'unsupported Rust: Rust type alias uses require an unimplemented provenance mapping' "$work/$case.log"
done
if RUSTC_BOOTSTRAP=1 "$adapter" "$fixtures/unstable.rs" "$work/bootstrap.c" > "$work/bootstrap.log" 2>&1; then
    echo "ambient bootstrap enabled unstable source" >&2
    exit 1
fi
test ! -e "$work/bootstrap.c"
grep -q 'E0554' "$work/bootstrap.log"
RUSTC_BOOTSTRAP=1 "$adapter" "$fixtures/model.rs" "$work/valid.c"
"$adapter" "$fixtures/scopes.rs" "$work/scopes.c"
# The nested tail block must retain braces at function-body indentation.
grep -q '^    {$' "$work/scopes.c"
grep -q '^        int32_t poly_v2 = poly_v1;$' "$work/scopes.c"
echo "ABI, nested scope, stable input and unchanged-output boundaries pass"
