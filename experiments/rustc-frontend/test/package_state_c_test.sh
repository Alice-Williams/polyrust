#!/usr/bin/env bash
set -euo pipefail
observed=$("$@")
printf '%s\n' "$observed"
count=$(grep -c '^C_PACKAGE_STATE_CHECKED' <<< "$observed")
test "$count" -eq 3
