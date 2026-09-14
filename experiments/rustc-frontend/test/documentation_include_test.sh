#!/usr/bin/env bash
set -euo pipefail
readonly adapter="$1"
readonly fixture="$2"
readonly work="${TEST_TMPDIR:?}/undeclared-documentation"
mkdir -p "$work"
cp "$fixture" "$work/input.rs"
for input_state in absent readable_but_undeclared; do
    if test "$input_state" = readable_but_undeclared; then
        printf 'explicitly declared documentation\n' > "$work/documentation.md"
    fi
    for state in absent existing; do
        output="$work/$input_state-$state.c"
        log="$work/$input_state-$state.log"
        if test "$state" = existing; then
            printf 'preserve existing artifact\n' > "$output"
            cp "$output" "$work/expected.c"
        fi
        if "$adapter" "$work/input.rs" "$output" > "$log" 2>&1; then
            echo "incorrectly admitted $input_state documentation" >&2
            exit 1
        fi
        grep -q 'documentation.md' "$log"
        if test "$input_state" = absent; then
            grep -q 'No such file or directory' "$log"
        else
            grep -q 'undeclared compiler file input:' "$log"
        fi
        if test "$state" = absent; then
            test ! -e "$output"
        else
            cmp "$output" "$work/expected.c"
        fi
    done
done
"$adapter" "$work/input.rs" "$work/declared.c" --input "$work/documentation.md"
grep -q 'explicitly declared documentation' "$work/declared.c"

# Environment-dependent attributes also need an explicit future dependency model.
printf '%s\n' '#[doc = env!("POLYRUST_DOCUMENTATION_VALUE")]' \
    'pub fn score(input: i32) -> i32 { input }' > "$work/environment.rs"
printf 'preserve existing artifact\n' > "$work/environment.c"
cp "$work/environment.c" "$work/environment.expected"
if POLYRUST_DOCUMENTATION_VALUE=known_test_value \
    "$adapter" "$work/environment.rs" "$work/environment.c" > "$work/environment.log" 2>&1; then
    echo 'incorrectly admitted environment-dependent documentation' >&2
    exit 1
fi
grep -q 'compiler environment-dependent input is not implemented' "$work/environment.log"
cmp "$work/environment.c" "$work/environment.expected"

# Bound extraction before cloning an oversized resolved attribute into metadata.
head -c 16777217 /dev/zero | tr '\0' x > "$work/documentation.md"
printf 'preserve existing artifact\n' > "$work/oversized.c"
cp "$work/oversized.c" "$work/oversized.expected"
if "$adapter" "$work/input.rs" "$work/oversized.c" --input "$work/documentation.md" \
    > "$work/oversized.log" 2>&1; then
    echo 'incorrectly admitted oversized source documentation' >&2
    exit 1
fi
grep -q 'source documentation extraction budget exceeded' "$work/oversized.log"
cmp "$work/oversized.c" "$work/oversized.expected"

# A source file named '-' must be opened as the declared file, not rustc stdin.
printf '%s\n' 'pub fn score(input: i32) -> i32 { input }' > "$work/-"
adapter_absolute="$(realpath "$adapter")"
(cd "$work" && "$adapter_absolute" - "$work/dash.c" < /dev/null)
grep -q 'poly_score' "$work/dash.c"
