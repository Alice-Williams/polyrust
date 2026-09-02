#!/usr/bin/env bash
set -euo pipefail

readonly runfiles="${RUNFILES_DIR:-$0.runfiles}"
readonly generator="$(find "${runfiles}" -path '*/examples/real-world/stdlib-is-negative-zero/generate' -print -quit)"
readonly compare="$(find "${runfiles}" -path '*/examples/real-world/stdlib-is-negative-zero/compare.mjs' -print -quit)"
readonly upstream_index="$(find "${runfiles}" -path '*/third_party/stdlib-is-negative-zero/index.js' -print -quit)"
readonly upstream_main="$(find "${runfiles}" -path '*/third_party/stdlib-is-negative-zero/main.js' -print -quit)"
test -n "${generator}" && test -n "${compare}" && test -n "${upstream_index}" && test -n "${upstream_main}"
readonly work="$(mktemp -d)"
trap 'rm -rf "${work}"' EXIT

"${generator}" "${work}/generated" >/dev/null
mkdir -p "${work}/upstream"
cp "${compare}" "${work}/compare.mjs"
cp "${upstream_index}" "${work}/upstream/index.js"
cp "${upstream_main}" "${work}/upstream/main.js"
export PATH=/usr/local/bin:/usr/bin:/bin
pushd "${work}/generated/typescript" >/dev/null
tsc
popd >/dev/null
node --enable-source-maps "${work}/compare.mjs"
