#!/usr/bin/env bash
set -euo pipefail

for tool in cargo rustfmt bazelisk node npm tsc prettier python3 ruff mypy pytest cargo-audit; do
    command -v "${tool}" >/dev/null || {
        echo "required release tool is missing: ${tool}" >&2
        exit 1
    }
done

cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo audit --deny warnings
bazelisk --batch test --test_output=errors //...
bazelisk --batch run //crates/conformance:polyrust-conformance -- --all-targets --determinism
bazelisk --batch test --test_output=errors //:release_gate

