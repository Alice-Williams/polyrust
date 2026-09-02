#!/usr/bin/env bash
set -euo pipefail

readonly runfiles="${RUNFILES_DIR:-$0.runfiles}"

check_blob() {
  local relative="$1"
  local expected="$2"
  local path
  path="$(find "${runfiles}" -path "*/third_party/has-flag/${relative}" -print -quit)"
  test -n "${path}"
  test "$(git hash-object "${path}")" = "${expected}"
}

check_blob "index.js" "cf60795caf5889f3b670101c22056fb3ebff2fb2"
check_blob "upstream.d.ts" "a10218d9145cdda02deabee7435d3436d770ec6d"
check_blob "upstream.test.js" "81563282c2ce8f493ea2dd0c09c0b4f620eaa170"
check_blob "upstream.test-d.ts" "d9fa211100740e62ae6f464a36cf99453ef6328a"
check_blob "package.json" "3040d95b2606b0c8382846bb9d65d30a61234aa2"
check_blob "LICENSE.MIT.txt" "fa7ceba3eb4a9657a9db7f3ffca4e4e97a9019de"
check_blob "README.md" "f28e3028adfc43f664db6b67aa821f66b4239e07"
