#!/usr/bin/env bash
set -euo pipefail
readonly runfiles="${RUNFILES_DIR:-$0.runfiles}"
readonly rustc="$(find -L "${runfiles}" -path '*/rust_toolchain/bin/rustc' -print -quit)"
if [[ -z "${rustc}" ]]; then
  echo "pinned Rust compiler missing from contract-test runfiles" >&2
  exit 1
fi
python3 "$1" "${rustc}" "$2" "$3" "${TEST_TMPDIR}"
