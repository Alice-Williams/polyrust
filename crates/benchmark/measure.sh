#!/usr/bin/env bash
set -euo pipefail

readonly runfiles="${RUNFILES_DIR:-$0.runfiles}"
readonly root="${runfiles}/${TEST_WORKSPACE}"
readonly benchmark="$(find "${root}" -path '*/crates/benchmark/generation_benchmark' -print -quit)"
test -n "${benchmark}"
python3 "${root}/crates/benchmark/measure.py" "${benchmark}"

