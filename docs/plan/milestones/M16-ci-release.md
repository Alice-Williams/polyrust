# M16 — Add CI matrix, determinism benchmark, and release gate

- Status: in-progress
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
