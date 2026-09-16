# Native Linux CI specification

- Status: accepted for M16B
- Last updated: 2026-09-16

## Purpose

GitHub-hosted Linux runners MUST execute Polyrust checks directly. The
development container exists to provide a Linux environment on non-Linux
developer hosts; CI is already Linux and MUST NOT build or run that container.

## Tool ownership

Bazel remains the authoritative build and test entry point. Rust, Go, Java,
and C/C++ compilation performed by Bazel MUST use the versions registered in
`MODULE.bazel` and `.bazelrc`.

Tools needed outside Bazel MUST be installed explicitly:

- Bazelisk is downloaded at a pinned version and verified against a
  repository-owned SHA-256 digest.
- Rust compatibility jobs install their selected rustup toolchain explicitly.
- The release job selects pinned Node and Python runtimes and installs pinned
  TypeScript, Prettier, Ruff, mypy, pytest, and cargo-audit versions.
- Sanitizer tests select GCC and G++ 14.2 explicitly but MUST NOT depend on a
  distribution-specific package revision.

Generated-package tests MUST resolve native tools from `PATH`. They MAY add
development-container paths as fallbacks, but MUST NOT replace a valid caller
`PATH` or force `RUSTUP_HOME`.

## Cache contract

Native Bazel caches live under `/var/tmp/polyrust-cache`, outside the
checkout and writable by both the Linux development container and GitHub
runners. Other native release tools live under the runner temporary cache
root. The requirements in `ci-cache.md` apply unchanged.

## Release behavior

The release script MUST NOT duplicate workspace Rustfmt, Clippy, or unit-test
commands already represented by authoritative Bazel targets. The Rust
compatibility matrix MUST compile and link all workspace tests with all features
using cargo test --no-run --workspace --all-features --locked and its explicitly
selected toolchain. It checks compiler compatibility, not native test execution:
Cargo does not provide Bazel test directories or declared compiler runfiles.

The release job MUST execute all tests through the unfiltered Bazel test //...
entry point. Missing native tools/runfiles MUST fail that gate. No test may be
ignored or silently skipped to make the compiler-compatibility matrix pass.
Compatibility and release failures MUST remain blocking; a successful compile-
only matrix is not evidence that runtime tests or the complete CI passed.

The hosted workflow MUST contain no `docker build` or `docker run`
invocation. The Windows job MAY continue validating that the development
container contract exists for non-Linux contributors.

## Required evidence

- A policy test rejects Docker execution, an unpinned bootstrap, a
  container-specific cache path, or missing native tool setup.
- Shell syntax, Buildifier, documentation, and actionlint checks pass.
- The tracked release suite passes in the Linux development container.
- A clean hosted workflow passes all jobs and its release job executes
  directly on Ubuntu.
