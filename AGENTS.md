# Development environment

- Use the Linux Dev Container in `.devcontainer/devcontainer.json` for all
  builds and tests on non-Linux hosts, including Windows.
- Do not run `cargo`, `rustc`, `go`, `bazel`, or `bazelisk` directly on the
  Windows host. The repository is bind-mounted at `/workspace` for local edits.
- Bazel is the authoritative build and test entry point. Cargo manifests may be
  retained for Rust ecosystem tooling, but a change is not verified until its
  Bazel targets pass.
- Rust and Go compiler versions are pinned in `MODULE.bazel`; do not replace
  them with host-discovered toolchains.
- The container deliberately receives no SSH keys or GitHub credentials. Use
  host Git for commits and pushes.
- Do not commit generated output, Bazel output links, caches, or image archives.

# Engineering workflow

- The milestone contract and dependency order live in `docs/plan/README.md`.
  Read the active milestone before implementation and cite its ID in commits.
- Commit and push after each completed milestone. Do not combine unrelated
  milestone completions in one checkpoint.
- Keep target-independent semantics out of concrete emitters.
- A backend consumes checked IR; it must never accept unchecked input through a
  safe public API.
- Unsupported constructs produce diagnostics rather than target-specific
  approximations.
- Generated Rust and Go must be compiled and tested in the container, not only
  compared as text.
