#!/usr/bin/env bash
set -euo pipefail

readonly runfiles="${RUNFILES_DIR:-$0.runfiles}"
readonly generator="$(find "${runfiles}" -path '*/examples/real-world/html-escaper/generate' -print -quit)"
test -n "${generator}"
readonly work="$(mktemp -d)"
trap 'rm -rf "${work}"' EXIT
"${generator}" "${work}/first" >/dev/null
"${generator}" "${work}/second" >/dev/null
diff -ru "${work}/first" "${work}/second"
rm -rf "${work}/second"
"${generator}" "${work}/second" >/dev/null
diff -ru "${work}/first" "${work}/second"
