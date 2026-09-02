# v0.1 release checklist

This checklist is evidence for a release decision; it does not publish or sign
anything. Every required item is release-blocking.

## PRD success measures

| Measure | Evidence | Status |
| --- | --- | --- |
| One example generates four packages without edits | [Author guide](author-guide.md), `//examples/models-and-validation:all` | Pass |
| Constants, contract/implementation, concrete/abstract dispatch, ten tests | [Compiled example](../examples/models-and-validation/src/lib.rs) | Pass |
| 50 semantic vectors agree in evaluator and all targets | [Conformance v0](conformance-v0.md), `//crates/conformance:all_targets_test` | Pass |
| Native formatter/static/build checks for every package | [Generated review guide](generated-code-review.md), native Bazel tests | Pass |
| Repeated generation is byte-identical | M14/M15 determinism tests and cross-host CI comparison | Pass |
| External backend needs no core target switch | [Backend author guide](backend-author-guide.md), external backend test | Pass |
| Unsupported features fail before filesystem mutation | [CLI v0](cli-v0.md) and M09 failure tests | Pass |
| 1,000 declarations under 2 s/512 MiB | [Benchmark evidence](benchmark-v0.1.md) | Pass |

## Milestone task definitions of done

| Task | Permanent evidence | Status |
| --- | --- | --- |
| [M00](plan/milestones/M00-development-environment.md) | Pinned Linux/Bazel foundation | Pass |
| [M01](plan/milestones/M01-workspace.md) | Workspace and dependency boundary gate | Pass |
| [M02](plan/milestones/M02-portable-ir.md) | Versioned strict IR and golden fixture | Pass |
| [M03](plan/milestones/M03-diagnostics.md) | Stable diagnostics and render snapshots | Pass |
| [M04](plan/milestones/M04-checker.md) | Resolver/type/capability checker tests | Pass |
| [M05](plan/milestones/M05-evaluator.md) | Debug/release evaluator corpus | Pass |
| [M06](plan/milestones/M06-rust-builder.md) | Typed builder and compile-fail proofs | Pass |
| [M07](plan/milestones/M07-document-writer.md) | Bounded deterministic writer tests | Pass |
| [M08](plan/milestones/M08-backend-api.md) | Checked-only backend contract kit | Pass |
| [M09](plan/milestones/M09-cli-output.md) | Safe atomic CLI output/failure tests | Pass |
| [M10](plan/milestones/M10-rust-backend.md) | Generated Rust native debug/release gate | Pass |
| [M11](plan/milestones/M11-typescript-backend.md) | Strict generated TypeScript native gate | Pass |
| [M12](plan/milestones/M12-python-backend.md) | Typed generated Python native gate | Pass |
| [M13](plan/milestones/M13-go-backend.md) | Generated Go native gate | Pass |
| [M14](plan/milestones/M14-conformance.md) | 50 vectors, faults, mismatch and determinism | Pass |
| [M15](plan/milestones/M15-examples-extensions.md) | Clean contributor walkthrough and extension proof | Pass |
| [M16](plan/milestones/M16-ci-release.md) | Local/hosted cold-warm matrix and policies | Pass |

## Release evidence

- Local focused M16 policies: pass, including all six deliberate failures.
- Local benchmark: pass; exact measurement is in the benchmark report.
- Local cache-cold and cache-warm release scripts: pass. The final cold Bazel
  run executed all 34 tests; the explicit release suite executed 17 tests.
- Rust 1.98.0 MSRV and the 2026-08-20 stable channel (also 1.98.0) pass the
  complete Cargo workspace and doctests.
- RustSec advisory scan: pass against 1,233 current advisories and all 26 locked
  crate dependencies.
- [Hosted workflow run 33447798324](https://github.com/Alice-Williams/polyrust/actions/runs/33447798324):
  pass at `0257f559dbc22ca97dbd49c78e29fa42c13428c1`. Windows contract,
  fast lint/tests, MSRV, stable, cross-host determinism, and cold/warm release
  jobs all passed.
- Cross-host artifacts: `manifests-ubuntu-22.04` (artifact 9778717397) and
  `manifests-ubuntu-24.04` (artifact 9778714315), 28,893 bytes each; the
  dependent byte-for-byte comparison passed.
- Hosted cache-cold complete gate: pass in four minutes. The immediately
  following cache-warm complete gate: pass in 42 seconds.

## Post-v0.1 architecture release evidence

- [M30 compositional target-language IR](plan/milestones/M30-compositional-language-ir.md):
  complete. Shared codegen plus Rust, TypeScript, JavaScript, Python, Go, Java,
  C++, and C are `Pass` in the
  [compliance ledger](language-ir-compliance.md).
- Local Linux-container evidence at the M30 release checkpoint: complete
  uncached repository gate 201/201 and dedicated uncached release gate 178/178,
  including Buildifier, Rustfmt, Clippy, target-native checks, public consumers,
  differential tests, C/C++ sanitizers, compile-fail role tests, and deliberate
  source-policy failure injection.
- [Hosted workflow run 33577166696](https://github.com/Alice-Williams/polyrust/actions/runs/33577166696):
  pass at `64cec7defbad6b61c56511fc5a986fdb1b08ecf2`, including a clean
  cache-cold complete gate and its cache-warm repeat.
