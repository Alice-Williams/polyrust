# M35-03A-02F-02B-03 — Certified Java constant fields and imports

- Status: planned
- Parent: [public constants](M35-03A-02F-02B-public-constants.md)
- Depends on: M35-03A-02F-02B-02

## Contract

Implement the [Java constant specification](../../specification/typed-generation/languages/java/rust-public-constants.md).
Reuse GeneratedValueId and primitive public static final fields. Extend bounded
source/dependency certificates, exact field ownership, readonly reads and
qualified dependency-value references without custom runtime helpers.

## Definition of done and tests

- Typed constants-only and mixed facades certify with exact field/value/owner
  inventories; missing/duplicate/nonstatic/nonfinal/mistyped fields reject.
- Strict Java21 separately compiled consumers agree with independent scalar
  truth; writes to exported constants fail compilation.
- Retargeted, forged, stale and linked-only/coupled value reference metadata
  reject; field resources and complete exports are measured/verified.
- Existing Java gates remain enabled. Full isolated Bazel/lint, review,
  ignored examples and scoped push; HIR support remains child04.
