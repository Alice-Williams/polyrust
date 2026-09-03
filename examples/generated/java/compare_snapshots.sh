#!/usr/bin/env bash
set -euo pipefail

if (( $# == 0 || $# % 2 != 0 )); then
  echo "expected snapshot/generated argument pairs" >&2
  exit 2
fi

while (( $# > 0 )); do
  readonly_snapshot="$1"
  generated="$2"
  shift 2
  if ! cmp --silent "${readonly_snapshot}" "${generated}"; then
    echo "generated Java snapshot drift: ${readonly_snapshot}" >&2
    diff --unified "${readonly_snapshot}" "${generated}" >&2 || true
    exit 1
  fi
done

echo "curated Java snapshots are byte-identical"
