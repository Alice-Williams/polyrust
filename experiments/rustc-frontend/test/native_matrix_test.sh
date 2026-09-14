#!/usr/bin/env bash
# Compile the certified compiler-generated artifact, not a hand-built AST fixture.
set -euo pipefail
readonly source="$1"
readonly reference="$2"
readonly consumer="$3"
readonly seeds="$4"
readonly zig="$5"
readonly work="${TEST_TMPDIR:?}/native-matrix"
mkdir -p "$work"
test "$(gcc-14 -dumpfullversion)" = "14.2.0"
cp "$seeds" "$work/inputs.txt"
for ((value=-4096; value<=4096; value++)); do
    printf '%s\n' "$value" >> "$work/inputs.txt"
done
"$reference" < "$work/inputs.txt" > "$work/expected.txt"
common=(
    -std=c17 -Wall -Wextra -Wpedantic -Werror
    -Wstrict-prototypes -Wmissing-prototypes
    -fno-fast-math -ffp-contract=off -fsigned-char -fno-short-enums
)
for optimization in 0 2; do
    for compiler in gcc zig asan ubsan; do
        command=(gcc-14)
        flags=()
        case "$compiler" in
            zig) command=("$zig") ;;
            asan) flags=(-fsanitize=address -fno-pie -no-pie) ;;
            ubsan) flags=(-fsanitize=undefined -fno-sanitize-recover=all -fno-pie -no-pie) ;;
        esac
        binary="$work/$compiler-o$optimization"
        "${command[@]}" "${common[@]}" "${flags[@]}" "-O$optimization" \
            "$source" "$consumer" -o "$binary"
        ASAN_OPTIONS=detect_leaks=1:halt_on_error=1 \
        UBSAN_OPTIONS=halt_on_error=1 \
            "$binary" < "$work/inputs.txt" > "$binary.txt"
        diff -u "$work/expected.txt" "$binary.txt"
    done
done
echo "8,204-input parity passed: pinned GCC/Zig and GCC ASan/UBSan, each at O0/O2"
