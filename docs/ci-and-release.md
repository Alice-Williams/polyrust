# CI and release gate

The required CI workflow is [`.github/workflows/ci.yml`](../.github/workflows/ci.yml).
It has five independent layers:

- a Windows Server 2025 checkout validates the exact pinned Linux development
  container contract without running toolchains on the Windows host;
- a fast Ubuntu 24.04 job runs Cargo Rustfmt, warning-denied Clippy, workspace
  tests, Bazel Buildifier, and the dependency-boundary linter;
- Rust 1.98.0 (MSRV) and current stable each run the complete Cargo workspace;
- Ubuntu 22.04 and 24.04 generate clean seven-target trees which a dependent job
  compares byte-for-byte; and
- the final Ubuntu job can run only after every prior job succeeds, installs
  pinned `cargo-audit` 0.22.2, and runs the entire release gate first with empty
  Bazel caches and then with the warmed caches. That complete gate includes the
  Bazel Rustfmt and Clippy aspects plus every native generated-code test.

Toolchains used for generation are pinned by the
[development image](../.devcontainer/Dockerfile) and `MODULE.bazel`; Go, Java,
and the authoritative Zig C/C++ driver are supplied hermetically by Bazel
rather than inherited from a runner. GCC 14.2 is version-checked in the
container and used only for sanitizer instrumentation.
Caches contain only
downloads and action results, are mounted from `RUNNER_TEMP` outside the
checkout, and therefore cannot become Bazel packages. Generation, native tests,
conformance, policy, and determinism commands still run on every gate and
cannot be converted into allowed skips.

## Local release command

Inside the Linux development container, install the pinned advisory checker and
run:

```sh
cargo install cargo-audit --locked --version 0.22.2
bash tools/release/release_gate.sh
```

The script requires every formatter and `cargo-audit`, runs Cargo formatting,
Clippy, workspace tests, and RustSec advisories, runs all Bazel tests, executes
all 50 conformance vectors with determinism enabled, and then runs the explicit
non-skippable Bazel release suite. A missing tool is a failure.

Policy tests scan the exact registry dependency/version allowlist, documented
licenses and toolchain pins, generated Rust unsafe surface, and generated Go
imports. The failure-injection suite proves snapshot drift, a conformance
failure, a missing formatter, skipped C++, unsafe generated Rust/Go, and a new
unreviewed dependency all make the gate fail.

## CI exercise procedure

Push a candidate commit or use `workflow_dispatch`, wait for all required jobs,
then record the immutable run URL and both cross-host manifest artifact names in
the [release checklist](release-checklist-v0.1.md). A failed or cancelled job is
not release evidence. The candidate must also have a clean local cold and warm
run using the checked-in container.
