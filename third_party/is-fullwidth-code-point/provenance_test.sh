#!/usr/bin/env bash
set -euo pipefail

readonly runfiles="${RUNFILES_DIR:-$0.runfiles}"

check_blob() {
  local relative="$1"
  local expected="$2"
  local path
  path="$(find "${runfiles}" -path "*/third_party/is-fullwidth-code-point/${relative}" -print -quit)"
  test -n "${path}"
  test "$(git hash-object "${path}")" = "${expected}"
}

check_blob "index.js" "671f97f760779075aa362ec41063e7a3a528b0e8"
check_blob "upstream.d.ts" "729d2020516f0b64dbcb8cb3443b7777b3c04769"
check_blob "upstream.type.test-d.ts" "6b7b42f49334ff0022f25d839d66b54a15e2e11f"
check_blob "upstream.test.js" "08d04261804685dc9e5a62f637f1727e99692cd3"
check_blob "package.json" "2137e888fa503dadf920e306c1cc12851a9de011"
check_blob "LICENSE.MIT.txt" "e7af2f77107d73046421ef56c4684cbfdd3c1e89"
check_blob "README.md" "4236bba980d8fea1486c883c16417ca8d5a7d5aa"
