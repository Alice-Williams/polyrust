# M00 — Establish the Linux/Bazel development baseline

- Status: complete
- Phase: 0
- Depends on: None

## Outcome

Provide one reproducible Linux container and one Bazel command that build and
test both Rust and Go, independent of the Windows host toolchain.

## Implementation checklist

- [x] Bind-mounted Dev Container derived from the official pinned Rust image.
- [x] Verified Bazelisk installation and repository-local Bazel version pin.
- [x] Bzlmod configuration with pinned `rules_rust`, Rust, `rules_go`, and Go SDKs.
- [x] Persistent but disposable Bazel/Cargo cache volumes.
- [x] Minimal Rust and Go library tests proving both toolchains work.
- [x] Bazel-native rustfmt, Clippy-with-warnings-denied, and Buildifier tests.
- [x] Developer documentation and agent instructions that make the container the
  authoritative non-Linux environment.

## Required exit evidence

- No host Rust, Go, Java, or Bazel installation is required.
- The source tree remains on the host and is visible at `/workspace`.
- The Go SDK is downloaded and selected by Bazel rather than discovered on
  `PATH`.
- Bazel output and caches are ignored by Git.
- The container has no host SSH key or GitHub credential mounts.
- Rebuilding without source/version changes reuses the named caches.

### Verification

Run inside a newly built container:

```text
bazelisk version
bazel test //...
bazel run @io_bazel_rules_go//go -- version
rustc --version
cargo --version
```

The Bazel test invocation MUST discover and pass at least one `rust_test`, one
`go_test`, rustfmt, Clippy, and Buildifier. Delete the Bazel output links and
rerun to prove the build is not dependent on untracked generated files.

Evidence recorded on 2026-08-31:

- The image built from `rust:1.98.0-trixie`; Bazelisk 1.29.0 selected Bazel 9.2.0.
- The Bazel-managed SDKs reported Rust 1.98.0 and Go 1.25.14 on Linux amd64.
- After deleting only the ignored Bazel output links, the cached rebuild passed.
- `bazel test //...` passed eight tests: native Rust and Go tests, generated
  Rust and Go tests, checker tests, rustfmt, Clippy with warnings denied, and
  Buildifier formatting/Starlark lint.

### Completion gate

The image builds from a clean Docker cache, all commands above pass, versions
match the checked-in pins, the Bzlmod lockfile is committed, and
`docs/DEVELOPMENT.md` is sufficient for a new contributor to reproduce the run.

## Scope boundary

Production-complete IR, checker, emitters, CI publication, registry releases,
and host Git credential setup. A deliberately thin vertical prototype may exist
as risk evidence without completing or superseding later milestones.
