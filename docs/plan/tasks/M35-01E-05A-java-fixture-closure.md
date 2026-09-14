# M35-01E-05A — Complete fixture matrix and inspectable examples

- Status: complete
- Parent: [M35-01E-05](M35-01E-05-java-native-proof.md)
- Depends on: M35-01E-04

## Implementation contract

- Audit the admitted entry fixtures (model, alternate, scopes, mapping inventory,
  documentation, direct calls, boolean order), public-package and same-spelling
  fixtures, and the real four-crate diamond. Explicitly distinguish metadata-only
  export-graph fixtures that contain unsupported constructs from admitted programs.
- Preserve existing 8,204-input Rust/C-O0/C-O2/Java entry tests and the diamond's
  8,204 x 16 independent native results. Extend the public-package/same-spelling
  Java proof from a handful of fixed values to the complete boundary corpus with
  independent native Rust references; retain corresponding C proof coverage.
- Expose declared Bazel generation targets for all admitted examples, including
  declared auxiliary Rust/doc inputs. Copy actual artifacts to ignored host-visible
  directories and verify bytes; never commit generated outputs.
- Keep all compiler-negative, typed-registry/capability, private visibility,
  ordering, source documentation and historical backend tests enabled.

## Definition of done and tests

- A documented matrix names source, generation target, native reference, input
  count and private/metadata checks for every admitted fixture.
- Each generated Java example compiles with pinned Java 21 warnings as errors.
  Complete corpus comparisons pass; docs/aliases/same-spelling identities and
  constructor/field/call order have explicit evidence.
- Example copies compare byte-for-byte with Bazel outputs. Full migration gate
  and a fresh independent scoped review pass before closure.

## Boundary

No new source language feature, heap support or claim of universal equivalence.
E05B still owns isolation of unrelated edits and proposed committed-tree proof.

## Implementation evidence

- `java_fixture_native_test` adds independent public-package and same-spelling
  native Rust references/consumers to the actual C/Java provider bundle actions.
  Both compare 8,204 rows, with eight and two output columns respectively, against
  Java 21, GCC 14.2 O0/O2 and Zig O0/O2, plus explicit independent fixture truth.
  Full source export maps, alias cycles and distinct same-spelled IDs reconcile.
- All seven `generate_java_<fixture>` actions now expose their declared outputs;
  their existing native tests compare these exact bytes with repeat invocations.
  Included source/doc inputs are declared. All historical proof targets remain.
- [The fixture matrix](../../specification/typed-generation/languages/java/rust-hir-fixtures.md)
  names exact artifacts, references and coverage, including unsupported
  metadata-only fixtures rather than presenting them as admitted examples.
- Thirteen actual artifact files were copied into ignored
  `generated/m35-java-fixtures-preview/` and byte-checked using `cmp`/`diff -rq`.
  The existing four-owner production example is preserved separately.
- Focused gate `e48cce30-3594-45b4-9056-24aff0e02157`: 5/5 tests pass.
- Full migration gate `6445dc5b-0311-4696-961a-12529ee432ef`: 373/373 tests
  across 487 targets pass, including release, frontend, C, Java, shared, Rust
  lint/format, Buildifier and documentation checks. This is working-tree proof;
  E05B still owns exact proposed-tree verification.
- Post-review documentation gate (the same full target set),
  `5c0cd366-05cc-4d10-966e-ee587833f32b`: 373/373 tests across 487 targets pass.
- Independent Sol Extra High `java_fixture_closure_review`: no remaining core
  findings after adding per-row evidence/counts and exact reproduction commands.
  Optional entry `javac` implicit-source flags are deferred: the current explicit
  source/consumer pair already compiles in isolated test output, while separately
  compiled bundle owners use the stricter empty-sourcepath contract. A proposed
  same-name fixture with different outputs is also deferred: the current Rust
  functions intentionally have identical semantics, and distinct declaration and
  target identities already have compiler-backed and manifest assertions. Neither
  suggestion identifies incorrect translation of the current source fixture.
- Fresh `java_fixture_final_review` found one independent inventory gap:
  cross-backend equality alone allowed the same unwanted extra alias in both
  manifests. Exact eight/six binding-key lists and per-module mutation rejection
  close it. Re-review found no remaining core issues.
- Full post-fix gate `ea1d1801-23a1-4217-8062-d1b24056c5ab`: 373/373 tests pass.
  Fresh Sol Extra High `java_fixture_inventory_final_review` independently
  confirmed the correction and wider fixture proof; no core or optional findings.
