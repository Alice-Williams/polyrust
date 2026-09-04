# Native Linux CI specification

- Status: accepted for M16B
- Last updated: 2026-09-04

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
compatibility matrix retains direct Cargo tests because it validates compiler
versions distinct from the Bazel production toolchain.

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
