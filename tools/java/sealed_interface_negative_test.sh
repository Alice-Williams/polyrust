#!/usr/bin/env bash
set -euo pipefail

readonly sources=("$@")
readonly runfiles="${RUNFILES_DIR:-$0.runfiles}"
readonly javac="$(find -L "${runfiles}" -path '*remotejdk21_linux/bin/javac' -print -quit)"
readonly classes="${TEST_TMPDIR}/sealed-negative-classes"
readonly diagnostics="${TEST_TMPDIR}/sealed-negative-javac.log"

if [[ -z "${javac}" ]]; then
  echo "hermetic Java 21 javac was not present in runfiles" >&2
  exit 1
fi

mkdir -p "${classes}"
if LC_ALL=C LANG=C "${javac}" --release 21 -Werror -Xlint:all -d "${classes}" \
  "${sources[@]}" >"${diagnostics}" 2>&1; then
  echo "hostile Java interface implementation unexpectedly compiled" >&2
  exit 1
fi

if ! grep -Eq "sealed|permits clause" "${diagnostics}"; then
  echo "hostile Java interface fixture failed for an unexpected reason" >&2
  cat "${diagnostics}" >&2
  exit 1
fi
