#!/usr/bin/env bash
set -euo pipefail

readonly runfiles="${RUNFILES_DIR:-$0.runfiles}"
readonly generator="$(find "${runfiles}" -path '*/examples/real-world/slash/generate' -print -quit)"
readonly compare="$(find "${runfiles}" -path '*/examples/real-world/slash/compare.mjs' -print -quit)"
readonly upstream="$(find "${runfiles}" -path '*/third_party/slash/upstream.js' -print -quit)"
test -n "${generator}" && test -n "${compare}" && test -n "${upstream}"
readonly work="$(mktemp -d)"
trap 'rm -rf "${work}"' EXIT

"${generator}" "${work}/generated" >/dev/null
cp "${compare}" "${work}/compare.mjs"
cp "${upstream}" "${work}/upstream.mjs"
export PATH=/usr/local/bin:/usr/bin:/bin
pushd "${work}/generated/typescript" >/dev/null
tsc
popd >/dev/null
node --enable-source-maps "${work}/compare.mjs"
