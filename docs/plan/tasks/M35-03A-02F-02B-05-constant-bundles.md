# M35-03A-02F-02B-05 — Multi-crate constant metadata and integration proof

- Status: complete
- Parent: [public constants](M35-03A-02F-02B-public-constants.md)
- Depends on: M35-03A-02F-02B-04

## Contract

Finish typed constant descriptions, lossless versioned C/Java manifests,
compiler-to-producer agreement and multi-crate generation under the
[shared source contract](../../specification/typed-generation/rust-public-constants.md).
Only certified producers can authorize imported public-value references;
metadata alone must not create authority.

## Definition of done and tests

- Real constants-only dependency, mixed root and transitive/alias fixtures
  compile separately and agree in Rust/C/Java; retain finite module bindings.
- Exact bool/i32/i64 values round-trip through explicit versioned schemas;
  stale type/value/export/owner/path metadata rejects before publication.
- Producer value mutation causes a dependent rebuild and fails old native truth;
  unsupported schemas and absent/existing outputs are checked atomically.
- Complete runtime-free file/API inventories and docs; all local/private
  constant proofs remain green. Full gate, fresh review, examples and push.
- Close parent02B/02 only when every child is complete; do not mark all
  JavaConstants value families or the overall runtime migration complete.

## Ordered implementation

1. [05A — Owned constant bundle publication](M35-03A-02F-02B-05A-owned-constant-bundles.md) — complete.
2. [05B — Authenticated foreign constant reads](M35-03A-02F-02B-05B-foreign-constant-reads.md) — complete.
3. [05C — Cross-crate export closure](M35-03A-02F-02B-05C-constant-alias-closure.md) — complete.

Each child has its own complete proof, reviewed checkpoint and push. Publishing
owned constants in a bundle does not authorize a foreign Rust read or re-export.
The existing atomic negative guards remain until their specific path has positive
compiler/target/native proof.

## Closure evidence

Owned publication (05A), authenticated reads (05B) and alias closure (05C)
have separate reviewed/gated checkpoints. The final
[05C-08 receipt](M35-03A-02F-02B-05C-08-end-to-end-closure.md) maps all proof
obligations, exact 741-test release trees, native and mutation controls, real
Bazel invalidation/restoration, and inspectable generated artifacts. Manifests
remain descriptions, not authority-bearing input. No wider constant family or
legacy runtime retirement is claimed.
