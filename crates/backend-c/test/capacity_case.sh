#!/usr/bin/env bash
set -euo pipefail
binary="$1"
case="$2"
shift 2
# Rust's test harness succeeds with zero matches. A renamed/missing case must
# fail this action, not silently disappear from the complete Bazel suite.
inventory=$("$binary" --list --exact "$case" --format terse)
if [ "$inventory" != "$case: test" ]; then
    printf 'Expected exactly one capacity test: %s\nActual: %s\n' "$case" "$inventory" >&2
    exit 1
fi
exec "$binary" --exact "$case" --nocapture "$@"
