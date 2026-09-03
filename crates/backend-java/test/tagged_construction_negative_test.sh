#!/usr/bin/env bash
set -euo pipefail

readonly sources=("$@")
readonly runfiles="${RUNFILES_DIR:-$0.runfiles}"
readonly javac="$(find -L "${runfiles}" -path '*remotejdk21_linux/bin/javac' -print -quit)"
readonly classes="${TEST_TMPDIR}/tagged-construction-negative-classes"
readonly diagnostics="${TEST_TMPDIR}/tagged-construction-negative-javac.log"

if [[ -z "${javac}" ]]; then
  echo "hermetic Java 21 javac was not present in runfiles" >&2
  exit 1
fi

mkdir -p "${classes}"
readonly javac_arguments=(
  --release
  21
  -Werror
  -Xlint:all
  -d
  "${classes}"
)
if LC_ALL=C LANG=C "${javac}" "${javac_arguments[@]}" "${sources[@]}" >"${diagnostics}" 2>&1; then
  echo "external raw tagged-value construction unexpectedly compiled" >&2
  exit 1
fi

readonly expected_diagnostics=(
  "PolyOption(boolean,T) has private access"
  "optionSome(T) is not public in Runtime"
  "PolyResult(boolean,T,PolyError) has private access"
  "ok(T) is not public in Runtime"
  "fail(String,String) is not public in Runtime"
  "PolyValueResult(boolean,T,E) has private access"
  "valueResultOk(T) is not public in Runtime"
  "valueResultErr(E) is not public in Runtime"
  "stringReplaceAll(String,String,String) is not public in Runtime"
  "checkedRemI32(int,int) is not public in Runtime"
  "semanticEqual(Object,Object) is not public in Runtime"
  "bytesOf(List<Integer>) is not public in Runtime"
)
for expected in "${expected_diagnostics[@]}"; do
  if ! grep -Fq "${expected}" "${diagnostics}"; then
    echo "missing expected raw tagged-construction rejection: ${expected}" >&2
    cat "${diagnostics}" >&2
    exit 1
  fi
done
