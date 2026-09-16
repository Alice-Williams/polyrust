# M16B — Native Linux CI

- Status: complete
- Phase: 5
- Depends on: M16A

## Outcome

Remove development-container construction from GitHub Actions and execute the
same cached Bazel release contract directly on Linux runners.

## Task sequence

1. [M16B-01 — migrate CI to native Linux execution](../tasks/M16B-01-native-linux-ci.md)
2. [M16B-02 — Rust compatibility and native test ownership](../tasks/M16B-02-rust-compatibility-boundary.md)
3. [M16B-03 — Java native suite execution budget](../tasks/M16B-03-java-native-suite-budget.md)

## Required exit evidence

- The native Linux specification is enforced by a tracked Bazel policy test.
- No Linux workflow step builds or runs Docker.
- Native bootstrap tools and cache compatibility inputs are pinned.
- Local lint, documentation, policy, and release targets pass.
- A hosted run passes all jobs and saves the refreshed native cache lineage.
- The completed checkpoint is committed and pushed.

## Hosted completion — 2026-09-16

[Run 35107402373](https://github.com/Alice-Williams/polyrust/actions/runs/35107402373)
succeeded at code checkpoint 081a796ce28ef429ba46ab787df284b7e88694d1.
All eight jobs passed, including both Rust compiler versions, Rust/Bazel lint,
both Ubuntu determinism jobs, cross-host manifest comparison, the Windows
contract and the native Linux release gate.

The release log records all 684 workspace tests executed and passed; the Java
backend suite completed in 260.8 seconds. The explicit conformance command
passed 50 cases with evaluator and all eight targets agreeing, including
repeated-generation manifest determinism. The final release suite passed all
265 tests (264 cached and one executed).

No compatible cache lineage was available on this run. After the successful
release gate, GitHub saved the new persistent cache under:

`polyrust-v4-Linux-8ebb32cfd266138681872cd9f849a5f585b5048112aaa6a89da07889f3b60aa1-35107402373-1`

This is hosted evidence for the implementation checkpoint, not merely a local
test result. The documentation-only closure records that completed checkpoint.

## Scope boundary

This milestone does not vendor external repositories, remove the Windows
development-container contract, or redesign language backends.
