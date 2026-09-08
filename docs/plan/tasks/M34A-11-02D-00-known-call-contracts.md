# M34A-11-02D-00 — Closed C library-call foundation

- Status: planned
- Depends on: M34A-11-02C

## Goal

Give the safety verifier one authoritative, non-forgeable source of library
signatures and obligations before it reasons about calls. This moves the
library-contract prerequisite out of 03; it does not implement linking.

## Definition of done

- Implement the exact initial inventory in c/known-call-contracts.md under
  dialect/catalogue. Closed identities own signatures, headers, native library
  requirements and operand-specific obligations; no caller-authored effects.
- Extend callable AST construction/reconstruction with a distinct known-call
  alternative. A generated function with the same spelling or prototype cannot
  acquire a known contract. Generated indirect provenance remains independent.
- Retain every argument, type, qualifier and actual result category. Known
  integer predicates return Int, not Bool; macros cannot become function-address
  values. No variadic, arbitrary foreign or raw-source call route is introduced.
- Reuse existing known object/constant identities and scalar model rather than
  authoring duplicate type metadata. Do not register library bodies as generated.
- Every new callable variant is visited by structural, lexical, completeness,
  initialization and later safety traversals. Metadata is an obligation, not
  successful ownership, range, initialization or resource evidence.
- Keep CDialect/shared bindings, header emission and native dependency projection
  in 03, and actual generated runtime function bodies in 05/06.

## Tests and proof

- Exhaustive expected signature/qualifier/result/header/library inventory.
- Positive exact calls; wrong arity/type/void-value use; same-signature generated
  substitution; crossed brands; private cached child mutation; macro-address
  category rejection and independent known-call reconstruction controls.
- Both pinned C compilers check the callable type/predicate-result inventory;
  repository-owned probes are not certified generated packages.
- Rustdoc compile-fail, Rust/Bazel lint, full cached tracked/release gates and
  eight-target deterministic conformance. Record actual labels and invocation IDs.

## Commit gate

Commit and push this completed slice with M34A-11-02D-00. No Supports advertisement,
verified package or legacy-source certification follows from this foundation.
