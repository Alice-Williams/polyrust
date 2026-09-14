# M35-01E-04D-02A — Java owner source-byte reservation

- Status: complete
- Parent: [M35-01E-04D-02](M35-01E-04D-02-java-bundle-inventory.md)
- Depends on: M35-01E-04D-01

## Implementation contract

- Expose conservative source-byte measurement only through an immutable
  JavaDependencyApi. Walk its admitted closed-owner AST without rendering text.
- Charge every emitted identifier/qualified-name occurrence, declaration,
  parameter, record component, statement and expression. Bound fixed syntax and
  indentation; include already escaped owner-bound documentation presentation.
- Use checked arithmetic and reject over-budget/unsupported measurement shapes.
  This reserves final source bytes, not all temporary renderer memory or JVM
  bytecode; those are separate resource concerns.

## Definition of done and tests

- Small/large/deep/long-name/documented/record/repeated-import fixtures render
  within their independently computed reservations. Exact and one-over limits
  plus arithmetic overflow fail transactionally.
- Actual compiler graph proof measures every owner before test rendering and
  checks final bytes against the reservation. Existing native equivalence passes.
- Target tests, Rust/Clippy/Bazel linters and compiler graph gate pass. Record
  evidence; manifest/index aggregation remains D02B.

## Completion evidence and review assessment

- Full migration/release gate `5cc1c416-2067-47d6-ac75-6e332bfbbae5` passes
  all 367 tests across 464 targets, including Rust/Clippy/Bazel and unchanged
  opaque-source policy checks. Java unit/native suite: 296 passed, 0 ignored.
- Actual four-crate Rust/Java proof still agrees for 8,204 inputs x 16 outputs;
  every owner is measured before rendering and its final UTF-8 bytes fit.
- Dedicated tests cover expanded/hostile docs, records, deep conditionals,
  large local/name inventories and repeated long qualified foreign calls.
  Exact/one-over/overflow additions and depth failures are transactional.
- Fresh read-only Sol Extra High review `java_source_reservation_review` found
  no correctness, authority, undercount or admitted-shape rejection issues.
  It independently traced all admitted shapes, spelling variants, documentation
  multiplicity and fixed syntax/indentation against the structural renderer.
- Two optional coverage suggestions are deferred, not unresolved defects:
  a public-API near-256-MiB fixture would supplement the exact arithmetic tests
  and variable-heavy rendering fixtures, but is not needed to repair a current
  limit defect; aggregate public bundle limits remain required in D02B. A new
  fixed-syntax maintenance oracle would be supplementary drift detection; the
  normative charging rule, supported-shape tests, compiler graph byte assertions
  and independent renderer audit already establish the current bound. Any future
  renderer/subset expansion must update this accounting and its tests together.
