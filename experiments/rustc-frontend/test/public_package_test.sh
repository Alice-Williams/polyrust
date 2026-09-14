#!/usr/bin/env bash
set -euo pipefail
readonly adapter="$1" fixture="$2" assertions="$3" zig="$4"
readonly reference="$5" seeds="$6"
readonly artifact="$7"
readonly work="${TEST_TMPDIR:?}/public-package"
mkdir -p "$work"
for iteration in one two three; do
    "$adapter" "$fixture" "$work/$iteration" --package
done
diff -ru "$work/one" "$work/two"
diff -ru "$work/one" "$work/three"
diff -ru "$artifact" "$work/one"
python3 "$assertions" "$work/one" "$work/consumer.c"
cp "$seeds" "$work/inputs.txt"
for ((value=-4096; value<=4096; value++)); do printf '%s\n' "$value" >> "$work/inputs.txt"; done
"$reference" < "$work/inputs.txt" > "$work/expected.txt"
sources=("$work/one/"*.c)
test "${#sources[@]}" = 1
test "$(gcc-14 -dumpfullversion)" = "14.2.0"
common=(-std=c17 -Wall -Wextra -Wpedantic -Werror -Wstrict-prototypes -Wmissing-prototypes
    -fno-fast-math -ffp-contract=off -fsigned-char -fno-short-enums -I "$work/one")
for optimization in 0 2; do
    for compiler in gcc zig asan ubsan; do
        command=(gcc-14)
        flags=()
        case "$compiler" in
            zig) command=("$zig") ;;
            asan) flags=(-fsanitize=address -fno-pie -no-pie) ;;
            ubsan) flags=(-fsanitize=undefined -fno-sanitize-recover=all -fno-pie -no-pie) ;;
        esac
        prefix="$work/$compiler-o$optimization"
        "${command[@]}" "${common[@]}" "${flags[@]}" "-O$optimization" -c "${sources[0]}" -o "$prefix-impl.o"
        nm --defined-only "$prefix-impl.o" | awk '$2 == "T" { print $3 }' | sort > "$prefix.symbols"
        diff -u "$work/consumer.symbols" "$prefix.symbols"
        if "${command[@]}" "${common[@]}" "${flags[@]}" "-O$optimization" -c "$work/consumer.private.c" -o "$prefix-private.o" > "$prefix-private.log" 2>&1; then
            echo 'private helper leaked into the public API' >&2
            exit 1
        fi
        grep -Eq 'implicit declaration|undeclared function' "$prefix-private.log"
        for order in header-first standards-first interleaved; do
            "${command[@]}" "${common[@]}" "${flags[@]}" "-O$optimization" -c "$work/consumer.$order.c" -o "$prefix-$order.o"
            "${command[@]}" "${flags[@]}" "$prefix-impl.o" "$prefix-$order.o" -o "$prefix-$order"
            (
                ulimit -s 1024
                ASAN_OPTIONS=detect_leaks=1:halt_on_error=1 UBSAN_OPTIONS=halt_on_error=1 "$prefix-$order" < "$work/inputs.txt" > "$prefix-$order.txt"
            )
            diff -u "$work/expected.txt" "$prefix-$order.txt"
        done
    done
done
if "$adapter" "$fixture" "$work/one" --package > "$work/existing.log" 2>&1; then
    echo 'existing package was overwritten' >&2
    exit 1
fi
grep -Fq 'package output already exists' "$work/existing.log"
diff -ru "$work/one" "$work/two"
echo '8,204-input Rust/C public API parity: eight native configurations, three include orders, 1 MiB stack, exact exports and private-header rejection passed'
