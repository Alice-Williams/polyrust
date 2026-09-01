#!/usr/bin/env bash
set -euo pipefail

readonly runfiles="${RUNFILES_DIR:-$0.runfiles}"

check_blob() {
  local relative="$1"
  local expected="$2"
  local path
  path="$(find "${runfiles}" -path "*/third_party/parse-ms/${relative}" -print -quit)"
  test -n "${path}"
  test "$(git hash-object "${path}")" = "${expected}"
}

check_blob_with_upstream_trailing_blank() {
  local relative="$1"
  local expected="$2"
  local path
  path="$(find "${runfiles}" -path "*/third_party/parse-ms/${relative}" -print -quit)"
  test -n "${path}"
  test "$({ cat "${path}"; printf '\n'; } | git hash-object --stdin)" = "${expected}"
}

check_blob "index.js" "48bc04d4d08c3c2342322d34b909826a2d0b3af9"
check_blob "upstream.d.ts" "cd0dd861b8b4933b6475741c72d3a922cc2e2fb9"
check_blob "upstream.type.test-d.ts" "9b6a3ee50f42bab161b1991f9bb75432be7a0db2"
check_blob_with_upstream_trailing_blank "upstream.test.js" "3661908385acead1aecbe2610f3cb1b3e1aa3b3e"
check_blob "package.json" "25e3674e412362307ded3ed841e9421efc96617a"
check_blob "LICENSE.MIT.txt" "fa7ceba3eb4a9657a9db7f3ffca4e4e97a9019de"
check_blob "README.md" "cbb5f2e725add50a75c21288cb6f3b885de71ecd"
