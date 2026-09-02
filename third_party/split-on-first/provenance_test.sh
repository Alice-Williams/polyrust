#!/usr/bin/env bash
set -euo pipefail

readonly runfiles="${RUNFILES_DIR:-$0.runfiles}"

check_blob() {
  local relative="$1"
  local expected="$2"
  local path
  path="$(find "${runfiles}" -path "*/third_party/split-on-first/${relative}" -print -quit)"
  test -n "${path}"
  test "$(git hash-object "${path}")" = "${expected}"
}

check_blob "index.js" "40382351b958e9295774e881b8f39be35b9d0b29"
check_blob "upstream.d.ts" "26aec885a837b46304a822de4313050c5a3180f2"
check_blob "upstream.test.js" "0080f4fad9ab1848af390082ffbe8983b92eb9f1"
check_blob "upstream.test-d.ts" "752b23817a0e87538fe7013f652dc289285ebdda"
check_blob "package.json" "e32f74d35a239ef5df42fa4adb4a3f3101870ff8"
check_blob "LICENSE.MIT.txt" "fa7ceba3eb4a9657a9db7f3ffca4e4e97a9019de"
check_blob "README.md" "f98ff3d2adbe8a25af6421cc38e9b6f1ef32f45a"
