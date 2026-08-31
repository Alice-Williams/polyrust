#!/usr/bin/env bash
set -euo pipefail
runfiles="${RUNFILES_DIR:-$0.runfiles}"
source_file="$(find "$runfiles" -path '*/test-generated/generated.go' -print -quit)"
go_bin="$(find "$runfiles" -path '*/bin/go' -print -quit)"
test -n "$source_file" && test -n "$go_bin"
package="$(dirname "$source_file")"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
cp -RL "$package/." "$work/"
export GOROOT="$(dirname "$(dirname "$go_bin")")"
export PATH="$GOROOT/bin:/usr/bin:/bin"
cd "$work"
gofmt -w .
test -z "$(gofmt -d .)"
go vet ./...
go test ./...
if grep -R -E '"(unsafe|reflect)"' -- *.go; then
  echo "forbidden unsafe/reflection import" >&2
  exit 1
fi
