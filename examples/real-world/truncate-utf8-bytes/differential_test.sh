#!/usr/bin/env bash
set -euo pipefail

readonly runfiles="${RUNFILES_DIR:-$0.runfiles}"
readonly generator="$(find "${runfiles}" -path '*/examples/real-world/truncate-utf8-bytes/generate' -print -quit)"
readonly compare="$(find "${runfiles}" -path '*/examples/real-world/truncate-utf8-bytes/compare.mjs' -print -quit)"
readonly upstream_index="$(find "${runfiles}" -path '*/third_party/truncate-utf8-bytes/index.js' -print -quit)"
readonly upstream_impl="$(find "${runfiles}" -path '*/third_party/truncate-utf8-bytes/lib/truncate.js' -print -quit)"
readonly upstream_corpus="$(find "${runfiles}" -path '*/third_party/truncate-utf8-bytes/blns.json' -print -quit)"
test -n "${generator}" && test -n "${compare}" && test -n "${upstream_index}"
test -n "${upstream_impl}" && test -n "${upstream_corpus}"
readonly work="$(mktemp -d)"
trap 'rm -rf "${work}"' EXIT

"${generator}" "${work}/generated" >/dev/null
mkdir -p "${work}/upstream/lib"
cp "${compare}" "${work}/compare.mjs"
cp "${upstream_index}" "${work}/upstream/index.js"
cp "${upstream_impl}" "${work}/upstream/lib/truncate.js"
cp "${upstream_corpus}" "${work}/upstream/blns.json"
export PATH=/usr/local/bin:/usr/bin:/bin
pushd "${work}/generated/typescript" >/dev/null
tsc
popd >/dev/null
node --enable-source-maps "${work}/compare.mjs"
