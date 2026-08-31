# Contributing

Development is Linux-first and Bazel-first. On Windows or macOS, run build and
test commands inside the checked-in Dev Container; do not use host Rust, Go, or
Bazel installations.

## Quality gate

From `/workspace` inside the container, run:

```bash
bazel test //...
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo metadata --format-version 1 --no-deps
```

`bazel test //...` is authoritative. It compiles generated Rust and Go with the
pinned Bazel toolchains and includes Rustfmt, Clippy, and Buildifier checks.
Cargo commands provide ecosystem-tooling compatibility but do not replace the
Bazel gate.

## Toolchains

- Rust's minimum supported version (MSRV) is 1.98.0 and is declared in the root
  Cargo manifest. It is also the pinned project Rust version for this phase.
- Bazel, Bazelisk, Rust, Go, and rule-set versions are pinned in repository
  configuration. Do not select compilers from the host `PATH`.
- Add Rust libraries as workspace dependencies and record every external
  dependency in `docs/dependencies.md`.

## Dependency direction

Dependencies point toward the semantic core. Core crates must not depend on the
CLI, conformance harness, or a concrete backend. Run
`//tools/dependency-boundaries:dependency_boundaries_test` through the normal
Bazel gate after changing Cargo manifests.

## Milestones and commits

Read `docs/plan/README.md` and the active milestone before starting. Keep changes
within that milestone's scope, cite its ID in the commit subject, and push each
completed milestone separately.
