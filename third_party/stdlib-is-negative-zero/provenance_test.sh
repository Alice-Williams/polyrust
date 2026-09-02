#!/usr/bin/env bash
set -euo pipefail

readonly runfiles="${RUNFILES_DIR:-$0.runfiles}"

upstream_path() {
  local relative="$1"
  find "${runfiles}" -path "*/third_party/stdlib-is-negative-zero/${relative}" -print -quit
}

check_blob() {
  local relative="$1"
  local expected="$2"
  local path
  path="$(upstream_path "${relative}")"
  test -n "${path}"
  test "$(git hash-object "${path}")" = "${expected}"
}

check_blob "index.js" "08012005cbc864c31a1cedbfb8e17612f3b2aaa5"
check_blob "main.js" "2b7e26a0a14d7c736e9ca59827c1b6230b5e4419"
check_blob "upstream.d.ts" "b5a2ef2b85b995d8cafead56b1c2af177785a2a4"
check_blob "upstream.type.test.ts" "e6479848f3fd7944e3b1e12520338173f42ad29f"
check_blob "upstream.test.js" "638d792ca314bbcbd874f2f15d54aee8405a331f"
check_blob "upstream.native.test.js" "452c96471825deba966978d9f82634f7a2001470"
check_blob "upstream.c" "4c22c9809fe27d7fa0b4f8264264551aab74daaa"
check_blob "upstream.h" "3d640db246e1651adcc766eae0cc683cc03cb6fd"
check_blob "package.json" "1d140ea9ce9aa72acf1737b8279c7b11b3443772"
check_blob "README.md" "dec48b10dd38243d205dab1f5349f7e9e6dd4cc3"
check_blob "NOTICE.txt" "995aba88721b9b7d761b11ec58cd44926209e27f"
check_blob "LICENSE.Apache-2.0.txt" "f433b1a53f5b830a205fd2df78e2b34974656c7b"

readonly package_json="$(upstream_path "package.json")"
readonly declaration="$(upstream_path "upstream.d.ts")"
grep -Fq '"name": "@stdlib/math-base-assert-is-negative-zero"' "${package_json}"
grep -Fq '"version": "0.2.3"' "${package_json}"
grep -Fq '"license": "Apache-2.0"' "${package_json}"
grep -Fq 'declare function isNegativeZero( x: number ): boolean;' "${declaration}"
