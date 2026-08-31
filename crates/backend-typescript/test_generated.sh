#!/usr/bin/env bash
set -euo pipefail

runfiles="${RUNFILES_DIR:-$0.runfiles}"
index="$(find "$runfiles" -path '*/test-generated/src/index.ts' -print -quit)"
test -n "$index"
package="$(dirname "$(dirname "$index")")"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
cp -RL "$package/." "$work/"
cd "$work"
export PATH="/usr/local/bin:/usr/bin:/bin"
prettier --write . >/dev/null
prettier --check .
tsc --noEmit
npm test
cp tests/invalid-types.ts src/invalid-types.ts
tsc --noEmit
if grep -R -E '\bi64[^\n]*number|\b(unsafe|reflect)\b' src/index.ts src/runtime.ts; then
  echo "forbidden generated construct" >&2
  exit 1
fi
