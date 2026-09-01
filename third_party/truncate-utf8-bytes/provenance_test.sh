#!/usr/bin/env bash
set -euo pipefail

readonly runfiles="${RUNFILES_DIR:-$0.runfiles}"

check_blob() {
  local relative="$1"
  local expected="$2"
  local path
  path="$(find "${runfiles}" -path "*/third_party/truncate-utf8-bytes/${relative}" -print -quit)"
  test -n "${path}"
  test "$(git hash-object "${path}")" = "${expected}"
}

check_blob "index.js" "39e899c37db279fb390eb9b477728155621502d4"
check_blob "lib/truncate.js" "3fed3b6f0cf161d2c43f952ba2d60a37702c4869"
check_blob "upstream.test.js" "bde24f4158267dbc04185e17b2c2344a70f6eb87"
check_blob "package.json" "3789bd7162f2722b31ddaa33e90946922cb0465b"
check_blob "LICENSE.MIT.txt" "d508675a1c7ee50fc7ac6a453306156199fbaf49"
check_blob "upstream.d.ts" "0d254978abd717ff312cbeac0d2473c7bcf2ede7"
check_blob "blns.json" "1dee499ec96f223341a33eee6808314655ec6c2d"
