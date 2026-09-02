#!/usr/bin/env bash
set -euo pipefail

readonly runtime_source="$1"
readonly invalid_source="$2"
readonly runfiles="${RUNFILES_DIR:-$0.runfiles}"
readonly javac="$(find -L "${runfiles}" -path '*remotejdk21_linux/bin/javac' -print -quit)"
readonly classes="${TEST_TMPDIR}/negative-classes"
readonly diagnostics="${TEST_TMPDIR}/negative-javac.log"

if [[ -z "${javac}" ]]; then
  echo "hermetic Java 21 javac was not present in runfiles" >&2
  exit 1
fi

mkdir -p "${classes}"
if "${javac}" --release 21 -Werror -Xlint:all -d "${classes}" \
  "${runtime_source}" "${invalid_source}" >"${diagnostics}" 2>&1; then
  echo "negative Java type fixture unexpectedly compiled" >&2
  exit 1
fi

if ! grep -q "incompatible types" "${diagnostics}"; then
  echo "negative Java fixture failed for an unexpected reason" >&2
  cat "${diagnostics}" >&2
  exit 1
fi
