# M01 — Scaffold workspace and quality baseline

- Status: complete
- Phase: 1
- Depends on: M00

## Outcome

Create the Rust workspace and crate boundaries from the technical specification,
with Bazel as the authoritative build/test entry point and Cargo manifests for
Rust ecosystem tooling.

## Implementation checklist

- Root Cargo workspace plus matching Bazel targets, with a documented minimum
  supported Rust version (MSRV).
- Empty/public-minimal crates for IR, diagnostics, checker, evaluator, builder,
  codegen, four backends, conformance harness, and CLI.
- Workspace lint policy that forbids unsafe code in core and generated-Rust
  support crates.
- `CONTRIBUTING.md` with local quality commands and toolchain policy.
- Dependency inventory documenting purpose and license.

## Required exit evidence

- Dependency direction is documented and enforced by crate dependencies.
- Core crates do not depend on CLI or concrete backends.
- All crates build on the selected MSRV and current stable Rust in CI or a local
  equivalent.
- No Git initialization or remote publication occurs in this task.

### Verification

```text
bazel test //...
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo metadata --format-version 1
```

Add a dependency-boundary test or script that fails if a core crate gains a
concrete backend dependency.

### Completion gate

All commands pass from a fresh checkout/workspace, crate-level documentation
states each responsibility, MSRV is tested, dependency inventory is reviewed,
and no placeholder test is the only test in any non-empty crate.

### Completion evidence

Completed in the pinned Linux development image on 2026-08-31:

- `bazel test //...` passed all 9 tests across 29 analyzed targets, including
  Buildifier, Rustfmt, Clippy with warnings denied, dependency-boundary policy,
  and native generated Rust and Go tests.
- `cargo fmt --all --check` passed.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
  passed for all 13 workspace packages.
- `cargo test --workspace --all-features` passed, including the existing checker
  behavior test; intentionally API-empty M01 boundary crates contain no
  placeholder tests.
- `cargo metadata --format-version 1` resolved only internal Cargo path
  dependencies and reported Rust 1.98.0 as every package's MSRV.
- `//tools/dependency-boundaries:dependency_boundaries_test` enforces that core
  Cargo manifests cannot depend on concrete backends, conformance, or CLI.
- `docs/dependencies.md` records all pinned build dependencies and confirms that
  M01 adds no third-party Rust runtime dependencies.

## Scope boundary

IR nodes, backend behavior, Git setup, publishing, and target toolchain installs.
