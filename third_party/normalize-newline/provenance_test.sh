#!/usr/bin/env bash
set -euo pipefail

readonly runfiles="${RUNFILES_DIR:-$0.runfiles}"

check_blob() {
  local relative="$1"
  local expected="$2"
  local path
  path="$(find "${runfiles}" -path "*/third_party/normalize-newline/${relative}" -print -quit)"
  test -n "${path}"
  test "$(git hash-object "${path}")" = "${expected}"
}

check_blob "index.js" "d5f7dabd319762ddeffa7e46789ff68d2df66fa3"
check_blob "upstream.d.ts" "4804149aa994a9bcfcc55efdd15c566c159b313f"
check_blob "upstream.test.js" "b50dbea911928b42eee0a28f476d2d8201a5ce03"
check_blob "package.json" "d210597b8aa7b8a21a32b8a992a007e9ccae127c"
check_blob "LICENSE.MIT.txt" "fa7ceba3eb4a9657a9db7f3ffca4e4e97a9019de"
check_blob "README.md" "81c6a38692853b0ffb3ad3c51a549d4bf18cdc97"
