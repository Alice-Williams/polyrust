# M35-01D-04C — Checked Rust dependency driver

- Status: complete
- Depends on: [M35-01D-04B](M35-01D-04B-certified-c-dependencies.md)
- Parent: [M35-01D-04](M35-01D-04-c-crate-linking.md)
- Contract: [crate dependencies](../../specification/typed-generation/languages/c/rust-hir-crate-dependencies.md)

## Goal

Join compiler-resolved foreign calls to separately checked C dependency APIs.

## Ordered implementation slices

1. [04C-01 — Pinned compiler metadata agreement](M35-01D-04C-01-metadata-agreement.md).
2. [04C-02 — Declared crate graph and compiler driver](M35-01D-04C-02-declared-crate-driver.md).
3. [04C-03 — Foreign typed bindings and package publication](M35-01D-04C-03-foreign-bindings.md).

First establish which pinned compiler evidence actually survives metadata
emission/loading; do not assume equal crate names or DefPathHashes prove equal
bodies. Keep that probe separate from production authority and do not enable
foreign calls until all source/metadata/certificate joins are implemented.

## Definition of done

- Pinned Bazel actions supply explicit per-crate configuration, source/doc and
  Rust metadata dependencies; no ambient lookup or arbitrary compiler flags.
- Reconstruct dependency certificates from declared inputs, then verify their
  compiler content/configuration identities against consumer-loaded metadata.
- Map foreign DefIds/signatures to exact certified imports; never lower a
  foreign body into the consumer implementation.
- Keep public aliases and ownership; manifests distinguish local definitions
  from imported foreign functions and verify complete membership both ways.
- All checks precede publication; failures preserve absent/existing output.

## Tests and proof

- Compiler probes inspect exact local/foreign identities, signatures and bindings.
- Illegal private access fails rustc; valid unmapped foreign APIs diagnose.
- Mismatched metadata/source/configuration/API inventory cannot bind by matching
  names or signatures alone; stale and coordinated substitutions reject.
- Declared-source/doc/metadata changes invalidate corresponding Bazel actions.
- Determinism and original single-crate/compiler-negative suites remain green.
- Review and Linux Bazel lint/target gates pass before native integration closure.

## Completion evidence

All three ordered slices are complete. The final bundle gate passed 340 tests
across 399 targets in `5195cc51-42df-4ffc-afc0-4d4dba87cf2d` with
fresh independent reviews finding no unresolved core issues. Actual action
inputs were checked in `8d24b854-5aa6-48a0-9c95-2180d49cfaac`.
Source/metadata agreement, exact target authority, checked typed imports and
whole-graph publication are integrated. The separate 04D native-equivalence
gate is still required before closing parent C integration and starting Java.
