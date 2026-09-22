# M35-03A-02V-04 — Checked signed-infinity source integration

- Status: planned
- Parent: [02V](M35-03A-02V-infinite-f64-constants.md)
- Depends on: [Java foundation](M35-03A-02V-03-java-constants.md)
- Specification: [shared](../../specification/typed-generation/rust-infinite-f64-constants.md)

## Contract

Add Infinity(Binary64Sign) only to the distinct checked constant domain.
Retain canonical compiler definition/context, exact type/width and provenance
guards. Exhaustively map read/local/public/import/alias paths. Preserve existing
literal restrictions and reject every NaN constant atomically.

## Definition of done and tests

Original multi-crate Rust and generated C/Java agree on exact infinity bits,
with source-owned declarations, original API/docs/privacy and alias identity.
Typed probes reject wrong values/signs/types/owners and compile-negative mapping
contracts hold. NaN payloads, f32 and unsupported source forms leave output
unchanged. Existing floating compositions preserve their documented semantics.
Actual Bazel producer-sign mutation invalidates affected metadata/packages and
native tests, leaves independent producers cached, and restoration recovers
cached passing results. Export real packages, update partial inventory, pass
full release/lint, preservation and fresh review before separate commit/push.
