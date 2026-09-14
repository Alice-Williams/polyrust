#!/usr/bin/env bash
set -euo pipefail
adapter="$1"
shift
"$adapter" "$@"
