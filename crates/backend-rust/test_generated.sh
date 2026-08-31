#!/usr/bin/env bash
set -euo pipefail

readonly repo_root="${TEST_SRCDIR}/${TEST_WORKSPACE}"
readonly generated="${repo_root}/crates/backend-rust/test-generated"
readonly package="${TEST_TMPDIR}/generated-rust"
readonly invalid="${TEST_TMPDIR}/invalid-rust"
readonly cargo_bin="/usr/local/cargo/bin/cargo"

export RUSTUP_HOME="/usr/local/rustup"
export CARGO_HOME="${TEST_TMPDIR}/cargo-home"
export PATH="/usr/local/cargo/bin:/usr/bin:/bin"

mkdir -p "${package}/src"
cp "${generated}/Cargo.toml" "${package}/Cargo.toml"
cp "${generated}/src/lib.rs" "${package}/src/lib.rs"
cp "${generated}/src/conformance.rs" "${package}/src/conformance.rs"
cp "${generated}/src/polyrust_runtime.rs" "${package}/src/polyrust_runtime.rs"

export CARGO_TARGET_DIR="${TEST_TMPDIR}/cargo-target"
"${cargo_bin}" fmt --manifest-path "${package}/Cargo.toml" --all
"${cargo_bin}" fmt --manifest-path "${package}/Cargo.toml" --all -- --check
"${cargo_bin}" clippy --manifest-path "${package}/Cargo.toml" --all-targets -- -D warnings
"${cargo_bin}" test --manifest-path "${package}/Cargo.toml"
"${cargo_bin}" test --manifest-path "${package}/Cargo.toml" --release

unsafe_count="$(grep -R -c 'unsafe' "${package}/src" | awk -F: '{ total += $2 } END { print total + 0 }')"
if [[ "${unsafe_count}" != "1" ]]; then
  echo "generated Rust contains an unexpected unsafe surface" >&2
  exit 1
fi

cp -R "${package}" "${invalid}"
printf '\npub unsafe fn deliberately_invalid_backend_artifact() {}\n' >> "${invalid}/src/lib.rs"
if "${cargo_bin}" check --manifest-path "${invalid}/Cargo.toml" >"${TEST_TMPDIR}/compile-fail.log" 2>&1; then
  echo "deliberately unsafe generated artifact unexpectedly compiled" >&2
  exit 1
fi
grep -q 'unsafe' "${TEST_TMPDIR}/compile-fail.log"
