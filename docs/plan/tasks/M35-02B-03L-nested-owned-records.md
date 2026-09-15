# M35-02B-03L — Nested owned record correspondence

- Status: in-progress
- Parent: [M35-02B-03](M35-02B-03-structured-owned-places.md)
- Depends on: M35-02B-03K-02
- Specification: [nested owned records](../../specification/typed-generation/languages/c/rust-nested-owned-records.md)

## Contract

Discharge G's required nested-record obligation with a separately specified
finite record tree, actual compiler nominal/field types and full typed Place
paths. Do not treat a Box containing a scalar record as evidence for a record
containing another record that owns Boxes. Existing flat readers stay closed.

## Ordered checkpoints

1. [L-01 — Pinned nested observations](M35-02B-03L-01-nested-observations.md):
   establish compiler aggregate staging, whole-inner transfer, nested partial
   moves, selected read and exact cleanup paths before safe admission.
2. [L-02 — Nested construction](M35-02B-03L-02-nested-construction.md):
   specify private nested construction inputs and executable capability
   bindings with missing/wrong/private compile-negative contracts.
3. [L-03 — Nested correspondence](M35-02B-03L-03-nested-correspondence.md):
   certify the bounded complete source/body relation, project canonical
   source operations/paths/exits to ordinary consumers and test corruptions.

## Definition of done and tests

- Explicitly bounded nominal record tree, no cycles/custom Drop/allocators;
  source evaluation order and declaration-indexed layout remain distinct.
- Every owning leaf has an authenticated constructor-rooted chain through
  intermediate record moves and nested field projections to read/drop/transfer.
- Remaining leaves clean in Rust recursive declaration order; moved leaves or
  whole inner values cannot also be dropped in the old container.
- Positive nested shapes, reversed initializers, whole-inner moves, shadowing
  and first/last/multiple leaf extractions have exact ordinary-consumer evidence.
- Wrong nominal depth/type/field/staging/owner/order, partial missing/double drop,
  source reuse and incomplete paths reject; invalid Rust never acquires evidence.
- Each child passes a fresh broad review and full exact-tree Linux/Bazel gate
  before its own commit/push. No target heap admission occurs in this stage.

## Progress

L-01 is complete: the five-body pinned observation inventory and three invalid
Rust/fourteen valid-source controls passed all 488 isolated tests and a fresh
clean review after six assertion/oracle repairs. L-02 and L-03 remain planned;
the observation does not close this parent or admit nested records to C/Java.
