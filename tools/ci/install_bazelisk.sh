#!/usr/bin/env bash
set -euo pipefail

readonly version="1.29.0"
readonly expected_sha256="5a408715e932c0250d28bd84555f12edbf70117de42f9181691c736eacc4a992"
readonly bin_dir="${POLYRUST_CI_BIN:-/var/tmp/polyrust-cache/bin}"
readonly binary="${bin_dir}/bazelisk"

mkdir -p "${bin_dir}"
if [[ ! -x "${binary}" ]] ||
  [[ "$(sha256sum "${binary}" | cut -d ' ' -f 1)" != "${expected_sha256}" ]]; then
  temporary="$(mktemp)"
  trap 'rm -f "${temporary}"' EXIT
  curl --fail --silent --show-error --location --output "${temporary}" "https://github.com/bazelbuild/bazelisk/releases/download/v${version}/bazelisk-linux-amd64"
  actual_sha256="$(sha256sum "${temporary}" | cut -d ' ' -f 1)"
  if [[ "${actual_sha256}" != "${expected_sha256}" ]]; then
    echo "Bazelisk checksum mismatch" >&2
    exit 1
  fi
  install -m 0755 "${temporary}" "${binary}"
fi

if [[ -n "${GITHUB_PATH:-}" ]]; then
  printf '%s\n' "${bin_dir}" >>"${GITHUB_PATH}"
fi
