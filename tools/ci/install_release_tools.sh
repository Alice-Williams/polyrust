#!/usr/bin/env bash
set -euo pipefail

: "${POLYRUST_CI_CACHE_ROOT:?POLYRUST_CI_CACHE_ROOT must name an external cache directory}"
: "${GITHUB_ENV:?GITHUB_ENV is required in GitHub Actions}"
: "${GITHUB_PATH:?GITHUB_PATH is required in GitHub Actions}"

readonly rust_version="1.98.0"
readonly typescript_version="7.0.2"
readonly prettier_version="3.9.6"
readonly ruff_version="0.16.5"
readonly mypy_version="2.3.1"
readonly pytest_version="9.1.1"
readonly cargo_audit_version="0.22.2"
readonly node_tools="${POLYRUST_CI_CACHE_ROOT}/node-tools"
readonly python_tools="${POLYRUST_CI_CACHE_ROOT}/python-tools"
readonly cargo_home="${POLYRUST_CI_CACHE_ROOT}/cargo-home"
readonly cargo_tools="${POLYRUST_CI_CACHE_ROOT}/cargo-tools"

if ! rustup toolchain list | grep -q "^${rust_version}-"; then
  rustup toolchain install "${rust_version}" --profile minimal --component clippy --component rustfmt
fi
rustup default "${rust_version}"
printf 'RUSTUP_HOME=%s\n' "$(rustup show home)" >>"${GITHUB_ENV}"

mkdir -p "${node_tools}" "${cargo_home}" "${cargo_tools}"
if [[ ! -x "${node_tools}/node_modules/.bin/tsc" ]] ||
  [[ "$("${node_tools}/node_modules/.bin/tsc" --version)" != "Version ${typescript_version}" ]] ||
  [[ ! -x "${node_tools}/node_modules/.bin/prettier" ]] ||
  [[ "$("${node_tools}/node_modules/.bin/prettier" --version)" != "${prettier_version}" ]]; then
  npm install --prefix "${node_tools}" --ignore-scripts "typescript@${typescript_version}" "prettier@${prettier_version}"
fi

if [[ ! -x "${python_tools}/bin/python3" ]]; then
  python3 -m venv "${python_tools}"
fi
if [[ "$("${python_tools}/bin/ruff" --version 2>/dev/null || true)" != "ruff ${ruff_version}" ]] ||
  [[ "$("${python_tools}/bin/mypy" --version 2>/dev/null || true)" != "mypy ${mypy_version} (compiled: yes)" ]] ||
  [[ "$("${python_tools}/bin/pytest" --version 2>/dev/null | head -n 1 || true)" != "pytest ${pytest_version}" ]]; then
  "${python_tools}/bin/pip" install "ruff==${ruff_version}" "mypy==${mypy_version}" "pytest==${pytest_version}"
fi

if [[ ! -x "${cargo_tools}/bin/cargo-audit" ]]; then
  CARGO_HOME="${cargo_home}" CARGO_INSTALL_ROOT="${cargo_tools}" cargo install cargo-audit --locked --version "${cargo_audit_version}"
fi

printf '%s\n' "${cargo_tools}/bin" "${node_tools}/node_modules/.bin" "${python_tools}/bin" >>"${GITHUB_PATH}"

test "$(gcc-14 -dumpfullversion)" = "14.2.0"
test "$(g++-14 -dumpfullversion)" = "14.2.0"
