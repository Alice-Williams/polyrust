#!/usr/bin/env bash
set -euo pipefail
readonly adapter="$1"
readonly native_rust="$2"
readonly native_c_o0="$3"
readonly native_c_o2="$4"
readonly fixtures="$(dirname "$5")"
readonly source="$5"
readonly extra_inputs=("${@:6}")
readonly work="${TEST_TMPDIR}/compiler-proof"
mkdir -p "$work"
cp "$fixtures/inputs.txt" "$work/inputs.txt"
for ((value=-4096; value<=4096; value++)); do
    printf '%s\n' "$value" >> "$work/inputs.txt"
done
"$native_rust" < "$work/inputs.txt" > "$work/rust.txt"
"$native_c_o0" < "$work/inputs.txt" > "$work/c-o0.txt"
"$native_c_o2" < "$work/inputs.txt" > "$work/c-o2.txt"
diff -u "$work/rust.txt" "$work/c-o0.txt"
diff -u "$work/rust.txt" "$work/c-o2.txt"
"$adapter" "$source" "$work/first.c" "${extra_inputs[@]}"
"$adapter" "$source" "$work/second.c" "${extra_inputs[@]}"
cmp "$work/first.c" "$work/second.c"
if grep -Eq '\bgoto\b|^[[:space:]]*[[:alnum:]_]+:' "$work/first.c"; then
    echo "structured output unexpectedly contains goto or a label" >&2
    exit 1
fi
grep -q 'if (' "$work/first.c"
grep -q '^[[:space:]]*else {$' "$work/first.c"
for case in use_after_move escaping_borrow conflicting_borrow wrong_type unsupported; do
    if "$adapter" "$fixtures/$case.rs" "$work/$case.c" > "$work/$case.log" 2>&1; then
        echo "incorrectly admitted $case" >&2
        exit 1
    fi
    test ! -e "$work/$case.c"
done
grep -q 'E0382' "$work/use_after_move.log"
grep -q 'E0515' "$work/escaping_borrow.log"
grep -q 'E0506' "$work/conflicting_borrow.log"
grep -q 'E0308' "$work/wrong_type.log"
grep -q 'unsupported Rust:' "$work/unsupported.log"
echo "Rust/C native parity, deterministic generation, compiler and admission negatives pass"
