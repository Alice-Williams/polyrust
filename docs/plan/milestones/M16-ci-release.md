# M16 — Add CI matrix, determinism benchmark, and release gate

- Status: complete
- Phase: 5
- Depends on: M01, M09, M10, M11, M12, M13, M14, M15

## Outcome

Make the v0.1 success criteria repeatable across supported hosts and toolchains
and collect evidence for a release decision.

## Implementation checklist

- Windows and Linux CI with pinned Rust, Node/TypeScript, Python, and Go versions.
- Fast PR jobs plus full four-target conformance/determinism jobs.
- Caching that never changes generated artifacts or skips semantic checks.
- 1,000-declaration generation benchmark and memory measurement procedure.
- Dependency/license/advisory checks and generated-source safety scans.
- Release checklist mapped to every PRD success measure and task DoD.

## Required exit evidence

- MSRV and current-stable Rust both pass the appropriate workspace tests.
- All supported target versions build and run generated packages.
- Go is provisioned in CI even if absent from a developer machine.
- Two clean jobs on different hosts generate byte-identical normalized manifests.
- The benchmark meets the PRD target or the PRD is revised with measured evidence
  before release.
- Release cannot pass with a skipped required backend or conformance vector.

### Verification

- CI configuration is exercised on a branch/temporary workflow before merge.
- Cache-cold and cache-warm runs both pass.
- Intentional snapshot drift, conformance failure, missing formatter, skipped Go,
  unsafe generated Rust/Go import, and dependency-policy violation each fail the
  expected job.
- Benchmark result includes tool version, host description, time, and peak memory.

```text
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo run -p polyrust-conformance -- --all-targets --determinism
```

### Completion gate

The complete matrix is green from clean environments, deliberate failure tests
are recorded, benchmark evidence is attached, release checklist has no unevidenced
items, and no job can silently downgrade a required four-target check to success.

## Scope boundary

Publishing crates/packages, signing releases, public repository creation, and
long-term performance optimization.

## Exit evidence

- [Hosted run 33447798324](https://github.com/Alice-Williams/polyrust/actions/runs/33447798324)
  passed from candidate commit
  `0257f559dbc22ca97dbd49c78e29fa42c13428c1`. Its Windows Server 2025
  contract, Rust 1.98.0 MSRV, current-stable Rust, fast lint/test, two clean
  generation, cross-host comparison, cache-cold release, and cache-warm release
  jobs all completed successfully.
- The Ubuntu 22.04 and Ubuntu 24.04 jobs uploaded
  `manifests-ubuntu-22.04` (artifact 9778717397) and
  `manifests-ubuntu-24.04` (artifact 9778714315), each 28,893 bytes. The
  dependent comparison required their complete trees to be byte-identical.
- The hosted cache-cold complete gate passed in four minutes and the subsequent
  cache-warm complete gate passed in 42 seconds. Caches live under
  `RUNNER_TEMP`, outside the checkout, and the release script executes every
  semantic test in both modes.
- The release script passed locally from both fresh named Docker volumes and
  warmed volumes. Its authoritative Bazel invocation passed all 34 tests, and
  its explicit non-skippable release suite passed all 17 tests, including
  Bazel Rustfmt, warning-denied Clippy, all four native generated packages,
  determinism, dependency boundaries, safety policy, and failure injection.
- The conformance command passed all 50 semantic cases and the portable test
  through the evaluator plus Rust, TypeScript, Python, and Go, with
  byte-identical repeated manifests.
- The six recorded failure injections prove that snapshot drift, conformance
  failure, a missing formatter, skipped Go, unsafe generated source, and an
  unreviewed dependency each fail the gate.
- The 1,000-declaration/four-target benchmark generated 1,073,079 bytes in
  265 ms with 11.09 MiB peak RSS, below the 2,000 ms and 512 MiB limits.
- The live RustSec scan checked 1,233 advisories against all 26 locked
  dependencies with no vulnerability failure. The workflow itself also passed
  actionlint 1.7.12 before its final candidate push.
