#!/usr/bin/env bash
set -euo pipefail

runfiles="${RUNFILES_DIR:-$0.runfiles}"
index="$(find "$runfiles" -path '*/test-generated-javascript/src/index.js' -print -quit)"
test -n "$index"
package="$(dirname "$(dirname "$index")")"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
cp -RL "$package/." "$work/"
cd "$work"
export PATH="/usr/local/bin:/usr/bin:/bin"
prettier --write . >/dev/null
prettier --check .
npm test
if find . -type f -name '*.ts' -print -quit | grep -q .; then
  echo "standalone JavaScript package contains TypeScript sources" >&2
  exit 1
fi
