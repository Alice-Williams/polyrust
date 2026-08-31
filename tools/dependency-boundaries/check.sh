#!/usr/bin/env bash
set -euo pipefail

readonly repo_root="${TEST_SRCDIR}/${TEST_WORKSPACE}"
readonly core_crates=(
  build
  check
  codegen
  diagnostics
  eval
  ir
)
readonly forbidden='polyrust-(backend-|cli|conformance)|portable_(backend_|cli|conformance)'

failed=0
for crate in "${core_crates[@]}"; do
  manifest="${repo_root}/crates/${crate}/Cargo.toml"
  if [[ ! -f "${manifest}" ]]; then
    echo "missing core manifest in runfiles: ${manifest}" >&2
    failed=1
    continue
  fi

  if matches="$(grep -En "${forbidden}" "${manifest}" || true)" && [[ -n "${matches}" ]]; then
    echo "forbidden outward dependency in crates/${crate}/Cargo.toml:" >&2
    echo "${matches}" >&2
    failed=1
  fi
done

if [[ "${failed}" -ne 0 ]]; then
  echo "core crates may not depend on concrete backends, conformance, or CLI" >&2
  exit 1
fi

echo "core dependency boundaries verified"
