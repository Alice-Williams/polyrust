# Dependency inventory

This inventory records dependencies that are downloaded or executed by the
project. M01 had no third-party Rust runtime/library dependencies. M02 adds the
two audited serialization dependencies below; Cargo and Bazel lock their full
transitive graphs.

## Rust library dependencies

| Dependency | Locked version | Purpose | License |
| --- | --- | --- | --- |
| `serde` | 1.0.229 | Strict derives for the versioned IR schema | Apache-2.0 OR MIT |
| `serde_json` | 1.0.151 | Bounded JSON decoding and deterministic compact encoding | Apache-2.0 OR MIT |

Neither dependency exposes filesystem, network, process, or target-language
behavior. Their transitive packages and checksums are recorded in `Cargo.lock`
and imported into Bazel through `rules_rust` crate universe.

## Build and development dependencies

| Dependency | Pinned version | Purpose | License |
| --- | --- | --- | --- |
| Bazel | 9.2.0 | Authoritative build and test runner | Apache-2.0 |
| Bazelisk | 1.29.0 | Selects and launches the pinned Bazel version | Apache-2.0 |
| Rust | 1.98.0 | Project implementation and generated-Rust toolchain | Apache-2.0 OR MIT |
| Go | 1.25.14 | Compiles and tests generated Go | BSD-3-Clause |
| Java | 21 language/runtime contract | Compiles and tests generated Java through Bazel's hermetic remote JDK | GPL-2.0 with Classpath Exception |
| Zig | hermetic SDK from `hermetic_cc_toolchain` 4.3.0 | Authoritative C17/C++20 compiler driver targeting glibc 2.17 | MIT |
| GCC | 14.2.0-19 | Secondary ASan/UBSan instrumentation in the Linux dev container; version asserted by every sanitizer test | GPL-3.0-or-later with GCC Runtime Library Exception |
| Node.js | 24.20.0 (Krypton LTS) | Executes generated TypeScript and its native test runner | MIT |
| npm | 11.19.0 (bundled with Node) | TypeScript package-manager contract | Artistic-2.0 |
| TypeScript | 7.0.2 | Strict generated-package type checking | Apache-2.0 |
| Prettier | 3.9.6 | Explicit generated TypeScript formatting post-process | MIT |
| Python | 3.13.5 | Executes and compiles generated Python packages | PSF-2.0 |
| Ruff | 0.16.5 | Formats and lints generated Python | MIT |
| mypy | 2.3.1 | Strict generated-Python type checking | MIT |
| pytest | 9.1.1 | Generated Python native test runner | MIT |
| `rules_rust` | 0.74.0 | Hermetic Bazel Rust rules and toolchain | Apache-2.0 |
| `rules_go` | 0.63.0 | Hermetic Bazel Go rules and toolchain | Apache-2.0 |
| `rules_java` | 9.6.1 | Hermetic Bazel Java rules and remote JDK toolchains | Apache-2.0 |
| `rules_cc` | 0.2.22 | Bazel C and C++ build/test rules | Apache-2.0 |
| `hermetic_cc_toolchain` | 4.3.0 | Hermetic multi-target Zig C/C++ toolchain | MIT |
| `rules_shell` | 0.8.0 | Runs repository policy tests under Bazel | Apache-2.0 |
| `buildifier_prebuilt` | 8.5.1.4 | Formats and lints Bazel/Starlark files | Apache-2.0 |
| `cargo-audit` | 0.22.2 | Release-time RustSec advisory scan | Apache-2.0 OR MIT |

Versions are sourced from `.bazelversion`, `.devcontainer/Dockerfile`, and
`MODULE.bazel`. `MODULE.bazel.lock` records Bazel module resolution. The Node
distribution is checksum-verified against Node's signed release directory, and
the exact TypeScript/Prettier versions are installed without lifecycle scripts.
CI installs the exact `cargo-audit` version with Cargo's locked dependency graph;
its live advisory database is never cached as semantic success.

## Internal Cargo dependency direction

```text
ir             diagnostics
 \                 /
  +---- check ----+
       /  |  \
  build  eval  codegen
                 |
             backends
                 |
            conformance
                 |
                cli
```

The diagram describes allowed inward direction, not a requirement that every
edge already exist. The CLI is the composition root. Core comprises IR,
diagnostics, check, eval, build, and codegen; none may depend on a concrete
backend, conformance, or CLI package.
