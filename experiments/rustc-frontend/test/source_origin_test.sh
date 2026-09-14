#!/usr/bin/env bash
set -euo pipefail
readonly probe="$1" source="$2" included="$3"
"$probe" "$source" --input "$included"
if "$probe" "$source" >"${TEST_TMPDIR:?}/undeclared-doc.log" 2>&1; then
    echo 'undeclared include_str! documentation unexpectedly admitted' >&2
    exit 1
fi
grep -F 'undeclared compiler file input:' "$TEST_TMPDIR/undeclared-doc.log"
grep -F 'documentation.md' "$TEST_TMPDIR/undeclared-doc.log"
