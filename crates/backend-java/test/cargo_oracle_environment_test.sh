#!/usr/bin/env bash
set -euo pipefail
readonly runfiles="${RUNFILES_DIR:-${TEST_SRCDIR}}"
readonly javac="$(find -L "${runfiles}" -path '*remotejdk21_linux/bin/javac' -print -quit)"
test -n "${javac}"
readonly java_home="$(dirname "$(dirname "${javac}")")"
readonly test_binary="$(realpath "$1")"
readonly case_name="tests::totality_names::typed_object_names_preserve_function_field_and_projection_identity"
"${test_binary}" --list --exact "${case_name}" | grep -Fx "${case_name}: test"
# Exercise the real oracle with Cargo's environment, not Bazel's test variables.
env -u RUNFILES_DIR -u TEST_SRCDIR -u TEST_TMPDIR \
  JAVA_HOME_21_X64="${java_home}" \
  "${test_binary}" --exact \
  "${case_name}" \
  --nocapture
