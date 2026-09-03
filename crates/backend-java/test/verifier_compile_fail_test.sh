#!/usr/bin/env bash
set -euo pipefail

readonly sources=("$@")
readonly runfiles="${RUNFILES_DIR:-$0.runfiles}"
readonly javac="$(find -L "${runfiles}" -path '*remotejdk21_linux/bin/javac' -print -quit)"

if [[ -z "${javac}" ]]; then
  echo "hermetic Java 21 javac was not present in runfiles" >&2
  exit 1
fi

readonly expected_diagnostics=(
  "variable result might not have been initialized"
  "variable value might not have been initialized"
  "variable text is already defined"
  "unreachable statement"
  "variable x might not have been initialized"
  "unreachable statement"
  "should be declared in a file named DifferentFilename.java"
)
if [[ "${#sources[@]}" -ne "${#expected_diagnostics[@]}" ]]; then
  echo "each invalid source must have one expected javac diagnostic" >&2
  exit 1
fi

for index in "${!sources[@]}"; do
  classes="${TEST_TMPDIR}/verifier-compile-fail-classes-${index}"
  diagnostics="${TEST_TMPDIR}/verifier-compile-fail-javac-${index}.log"
  mkdir -p "${classes}"
  if LC_ALL=C LANG=C "${javac}" \
    --release 21 \
    -Werror \
    -Xlint:all \
    -d "${classes}" \
    "${sources[index]}" >"${diagnostics}" 2>&1; then
    echo "invalid Java verifier counterexample unexpectedly compiled: ${sources[index]}" >&2
    exit 1
  fi
  expected="${expected_diagnostics[index]}"
  if ! grep -Fq "${expected}" "${diagnostics}"; then
    echo "missing expected javac rejection: ${expected}" >&2
    cat "${diagnostics}" >&2
    exit 1
  fi
done
