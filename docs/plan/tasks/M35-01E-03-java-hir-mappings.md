# M35-01E-03 — Java HIR mappings and certified single-crate output

- Status: complete
- Parent: [M35-01E](M35-01E-java-rustc-retrofit.md)
- Depends on: M35-01E-02

## Goal

Lower successful rustc analysis directly into the existing typed Java AST.

## Implementation

First complete [M35-01E-03A](M35-01E-03A-shared-compiler-origins.md), the
source-owned compiler metadata extraction and independent boundary proof.
Then implement [M35-01E-03B](M35-01E-03B-java-source-mappings.md), the ten
Java-owned mappings, their exact builder contracts and compiler/native probes.

- Add java_lower with one executable mapping per shared source capability and
  a consuming Java builder enforcing each mapping's exact context/output.
- Reuse compiler input/configuration/provenance extraction, not C AST assembly.
- Extract the existing compiler origin/module/export reader into a shared
  source-owned module, preserving budgets and source/doc dependency checks.
  Leave CDeclarationKey construction in C; Java consumes shared RustSourceOrigin.
- Represent i32/bool, private immutable scalar-field records and shared borrows
  explicitly; retain source-order evaluation before constructor-field ordering.
- Implement local bindings/scopes, comparisons, terminal branches, scalar
  function signatures and same-crate calls. Keep unsupported HIR diagnostic.
- Rust boolean ordering maps through typed boolean-to-int conditional nodes
  (false = 0, true = 1), because Java has no relational boolean operators.
  Evaluate each operand exactly once in source order; equality remains direct.
  Test all boolean operand pairs for all six comparison operators.
- Assemble actual public exports, private helpers and docs into the source-owned
  facade; run existing Java verification/link/resource certification.
- Expose checked Java output through the adapter and a declared Bazel action.

## Definition of done and tests

- Compiler-backed assertions inspect types, field identity/order, shared-borrow
  representation, scope, origins, effective visibility and derived imports.
- All ten Java mapping slots have positive and missing/duplicate/wrong-category,
  wrong-context/output compile-negative coverage.
- Same source fixtures execute against native Rust/C/Java over boundary and
  differential vectors. No fake CoreProgram, raw emitter or alternate certificate.
- Invalid Rust and valid unsupported Rust preserve absent/existing outputs.
- Full historical gates and fresh broad review pass before completion.

## Exit evidence

Both child checkpoints are complete. M35-01E-03A preserved generated C bytes and
passed its independent source-boundary review. M35-01E-03B implements all ten
Java mappings, typed contracts, AST/native/compiler negatives and shared bounded
admission. Final full gate `6d9d75bc-7588-4b40-8a5a-5a149c2e4f9b` passed 363/363;
fresh reviewer `java_hir_postbudget_review` returned no findings after all core
repairs. The detailed evidence and remaining E04/E05 boundaries are recorded in
the child tasks. This is single-crate completion, not whole-migration completion.
