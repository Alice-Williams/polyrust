# M16B-01 — Migrate CI to native Linux execution

- Status: in-progress
- Depends on: M16A-02
- Blocks: return to M34A language work

## Goal

Eliminate repeated development-image builds and execute Linux CI directly with
Bazel-managed toolchains plus a minimal pinned native bootstrap.

## Definition of done

- Fast checks, determinism generation, and the release gate call Bazelisk
  directly on their Ubuntu runners.
- Rust compatibility invokes explicitly selected rustup toolchains directly.
- The release job installs pinned native tools without Docker.
- Generated-package tests preserve caller `PATH` and do not force
  development-container Rust state.
- Native cache paths remain outside the checkout and Bazel test-result caching
  remains enabled.

## Tests

- Run shell syntax checks for the bootstrap, release, generated-package, and
  sanitizer scripts.
- Run the cache/native-CI policy test, Buildifier, documentation test,
  Rustfmt, and warning-denied Clippy.
- Run the tracked release suite in the Linux development container.
- Run actionlint 1.7.12 over the workflow.
- Push and prove all hosted jobs, the single native release gate, and cache
  save succeed.

## Commit gate

Commit and push only after local verification. Mark complete after hosted
evidence is recorded.

## Local evidence

- Pinned actionlint 1.7.12 accepts the native workflow.
- Shell syntax checks pass for every changed bootstrap, release, generated
  package, and sanitizer script.
- The native bootstrap verifies pinned Bazelisk by checksum, restores the
  cached release tools, and exports the selected Rust toolchain home.
- The complete tracked release suite passes 237 of 237 tests in the Linux
  development container, including Rustfmt, warning-denied Clippy,
  Buildifier, generated-package compilation, and language linters.
- An immediate cached replay also passes 237 of 237 tests while Bazel executes
  only Buildifier and reuses the other 236 recorded test results.
