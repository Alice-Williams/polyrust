#!/usr/bin/env bash
set -euo pipefail

runfiles="${RUNFILES_DIR:-$0.runfiles}"
typescript="$(find "$runfiles" -path '*/crates/backend-typescript/src/runtime.ts' -print -quit)"
javascript="$(find "$runfiles" -path '*/crates/backend-typescript/src/runtime.js' -print -quit)"
test -n "$typescript"
test -n "$javascript"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
export PATH="/usr/local/bin:/usr/bin:/bin"
tsc "$typescript" \
  --target ES2024 \
  --module ES2022 \
  --moduleResolution Bundler \
  --skipLibCheck \
  --outDir "$work"
cmp "$work/runtime.js" "$javascript"
