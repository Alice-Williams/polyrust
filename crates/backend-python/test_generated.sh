#!/usr/bin/env bash
set -euo pipefail

runfiles="${RUNFILES_DIR:-$0.runfiles}"
module="$(find "$runfiles" -path '*/test-generated/src/generated_polyrust/__init__.py' -print -quit)"
test -n "$module"
package="$(dirname "$(dirname "$(dirname "$module")")")"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
cp -RL "$package/." "$work/"
cd "$work"
export PATH="/opt/polyrust-python-tools/bin:/usr/local/bin:/usr/bin:/bin"
export PYTHONPATH="$work/src"
export MYPYPATH="$work/src"
python3 -m compileall -q src tests
ruff format .
ruff format --check .
ruff check .
mypy --strict src tests
if mypy --strict negative >/dev/null 2>&1; then
  echo "invalid Option tag unexpectedly type-checked" >&2
  exit 1
fi
pytest -q
if grep -E '^def [a-zA-Z][a-zA-Z0-9_]*\([^)]*(list\[|Any)|^def [a-zA-Z][a-zA-Z0-9_]*.* -> (list\[|Any)' src/generated_polyrust/__init__.py; then
  echo "forbidden public mutable/untyped API" >&2
  exit 1
fi
