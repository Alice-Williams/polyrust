#!/usr/bin/env bash
set -euo pipefail

readonly runfiles="${RUNFILES_DIR:-$0.runfiles}"

upstream_path() {
  local relative="$1"
  find "${runfiles}" -path "*/third_party/stdlib-abs/${relative}" -print -quit
}

check_blob() {
  local relative="$1"
  local expected="$2"
  local path
  path="$(upstream_path "${relative}")"
  test -n "${path}"
  test "$(git hash-object "${path}")" = "${expected}"
}

check_blob "index.js" "ed96b767e643b392aa734b4838df1c9e9a5c23f5"
check_blob "main.js" "8446df76227f89817742d4facc8725568a40f8db"
check_blob "bitwise.js" "0953c86bb16fd49dc8dd726a346e1fbeef770626"
check_blob "high.js" "cfa0cb86e704a0b4e16baac02c428f1f8f02bc22"
check_blob "native.js" "a24aecba263ba374d8d4a44a9cbaaf4e345d0df5"
check_blob "upstream.d.ts" "3662df94185b1e3f0a0174ca33fac8174bccc838"
check_blob "upstream.type.test.ts" "b369d726d573f83be02412cb63a559ae8ca1ebca"
check_blob "upstream.test.js" "26e66a4466c55528736171084f3616665ce07fb4"
check_blob "upstream.bitwise.test.js" "70ddc6eb82bf7096d650d2fdcebe160348a49fbc"
check_blob "upstream.high.test.js" "238b8379b375aa73874e36d0ac30c5715d1a6fa9"
check_blob "upstream.native.test.js" "5098d00a20d1844ed58c66e20037359eed8e7708"
check_blob "upstream.c" "94a078645b9e17d238e299db39a75c2950462970"
check_blob "upstream.h" "fcdba5f84f5f7a58d92fd12e39cd4eaec86bac45"
check_blob "package.json" "3c4fddb09061f23eb82e6b6f243713f760d2ed22"
check_blob "README.md" "6a7e7f720ebfd5d19b2d8bb652f0e798cdf1d51b"
check_blob "NOTICE.txt" "995aba88721b9b7d761b11ec58cd44926209e27f"
check_blob "LICENSE.Apache-2.0.txt" "f433b1a53f5b830a205fd2df78e2b34974656c7b"

readonly package_json="$(upstream_path "package.json")"
readonly declaration="$(upstream_path "upstream.d.ts")"
readonly main_source="$(upstream_path "main.js")"
readonly bitwise_source="$(upstream_path "bitwise.js")"
readonly c_source="$(upstream_path "upstream.c")"
grep -Fq '"name": "@stdlib/math-base-special-abs"' "${package_json}"
grep -Fq '"version": "0.2.3"' "${package_json}"
grep -Fq '"license": "Apache-2.0"' "${package_json}"
grep -Fq 'declare function abs( x: number ): number;' "${declaration}"
grep -Fq 'return Math.abs( x );' "${main_source}"
grep -Fq 'UINT32_VIEW[ HIGH ] &= ABS_MASK;' "${bitwise_source}"
grep -Fq 'w.words.high &= STDLIB_CONSTANT_FLOAT64_HIGH_WORD_ABS_MASK;' "${c_source}"
