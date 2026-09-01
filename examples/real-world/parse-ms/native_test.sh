#!/usr/bin/env bash
set -euo pipefail

readonly runfiles="${RUNFILES_DIR:-$0.runfiles}"
readonly rust_manifest="$(find "${runfiles}" -path '*/examples/real-world/parse-ms/generated/rust/Cargo.toml' -print -quit)"
readonly go_bin="$(find "${runfiles}" -path '*/bin/go' -print -quit)"
test -n "${rust_manifest}" && test -n "${go_bin}"
readonly generated="$(dirname "$(dirname "${rust_manifest}")")"
readonly work="$(mktemp -d)"
trap 'rm -rf "${work}"' EXIT
cp -RL "${generated}/." "${work}/"

export RUSTUP_HOME=/usr/local/rustup
export CARGO_HOME="${work}/cargo-home"
export CARGO_TARGET_DIR="${work}/cargo-target"
export PATH=/usr/local/cargo/bin:/opt/polyrust-python-tools/bin:/usr/local/bin:/usr/bin:/bin
cargo fmt --manifest-path "${work}/rust/Cargo.toml" --all
cargo fmt --manifest-path "${work}/rust/Cargo.toml" --all -- --check
cargo clippy --manifest-path "${work}/rust/Cargo.toml" --all-targets -- -D warnings
cargo test --manifest-path "${work}/rust/Cargo.toml"

pushd "${work}/typescript" >/dev/null
prettier --write . >/dev/null
prettier --check .
tsc --noEmit
npm test
popd >/dev/null

pushd "${work}/javascript" >/dev/null
prettier --write . >/dev/null
prettier --check .
npm test
test -z "$(find . -type f -name '*.ts' -print -quit)"
popd >/dev/null

pushd "${work}/python" >/dev/null
export PYTHONPATH="${work}/python/src"
export MYPYPATH="${work}/python/src"
python3 -m compileall -q src tests
ruff format .
ruff format --check .
ruff check .
mypy --strict src tests
pytest -q
popd >/dev/null

export GOROOT="$(dirname "$(dirname "${go_bin}")")"
export PATH="${GOROOT}/bin:/usr/bin:/bin"
pushd "${work}/go" >/dev/null
gofmt -w .
test -z "$(gofmt -d .)"
go vet ./...
go test ./...
popd >/dev/null
