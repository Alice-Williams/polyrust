#!/usr/bin/env bash
set -euo pipefail
readonly bundle="$1" reference="$2" seeds="$3" zig="$4" consumer="$5" symbols="$6"
readonly work="${TEST_TMPDIR:?}/crate-native"
mkdir -p "$work"
python3 "$consumer" "$bundle" "$work"
cp "$seeds" "$work/inputs.txt"
for ((value=-4096; value<=4096; value++)); do printf '%s\n' "$value" >> "$work/inputs.txt"; done
"$reference" < "$work/inputs.txt" > "$work/expected.txt"
test "$(gcc-14 -dumpfullversion)" = "14.2.0"
common=(-std=c17 -Wall -Wextra -Wpedantic -Werror -Wstrict-prototypes -Wmissing-prototypes
    -fno-fast-math -ffp-contract=off -fsigned-char -fno-short-enums
    -fno-inline -fno-optimize-sibling-calls -I "$bundle")
sources=("$bundle/"*.c)
test "${#sources[@]}" = 4
for optimization in 0 2; do
    for compiler in gcc zig asan ubsan; do
        command=(gcc-14)
        flags=()
        case "$compiler" in
            zig) command=("$zig") ;;
            asan) flags=(-fsanitize=address -fno-pie -no-pie) ;;
            ubsan) flags=(-fsanitize=undefined -fno-sanitize-recover=all -fno-pie -no-pie) ;;
        esac
        round="$work/$compiler-o$optimization"
        mkdir "$round"
        objects=()
        for source in "${sources[@]}"; do
            object="$round/$(basename "$source").o"
            "${command[@]}" "${common[@]}" "${flags[@]}" "-O$optimization" -c "$source" -o "$object"
            python3 "$symbols" "$work/members.json" "$(basename "$source")" "$object" "$optimization"
            objects+=("$object")
        done
        for private_source in "$work"/private-*.c; do
            private="$(basename "$private_source" .c)"
            if "${command[@]}" "${common[@]}" "${flags[@]}" "-O$optimization" -c "$private_source" -o "$round/$private.o" > "$round/$private.log" 2>&1; then
                echo "private $private leaked through a public header" >&2
                exit 1
            fi
            grep -Eq 'implicit declaration|undeclared function|incomplete type|storage size' "$round/$private.log"
        done
        for order in 0 1 2; do
            "${command[@]}" "${common[@]}" "${flags[@]}" "-O$optimization" -c "$work/consumer$order.c" -o "$round/consumer$order.o"
            "${command[@]}" "${flags[@]}" "${objects[@]}" "$round/consumer$order.o" -o "$round/run$order"
            (
                ulimit -s 1024
                test "$(ulimit -s)" = 1024
                ASAN_OPTIONS=detect_leaks=1:halt_on_error=1 UBSAN_OPTIONS=halt_on_error=1 "$round/run$order" < "$work/inputs.txt" > "$round/result$order.txt"
            )
            diff -u "$work/expected.txt" "$round/result$order.txt"
        done
    done
done
echo 'Rust/C four-crate diamond: 8,204 inputs, 16 results per input, 8 native profiles, 3 include orders, exact object ownership and private rejection passed'
