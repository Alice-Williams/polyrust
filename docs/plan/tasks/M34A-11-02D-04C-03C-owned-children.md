# M34A-11-02D-04C-03C — Owned children and partial rollback

- Status: planned
- Depends on: M34A-11-02D-04C-03B

## Goal

Compose initialized representation with exclusive, complete owned descendants.

## Definition of done

- Typed nominal/member/element child roles distinguish required/optional owners
  from borrowed metadata; actual active payload selects the inventory.
- Commit requires complete exclusive children and exact allocator provenance;
  shallow pointer copies, duplicate parents and owning cycles reject.
- Actual child transfer clears its previous slot; clone needs independent
  construction. Unproved generated helper effects remain 05 diagnostics.
- Every failed/early/cleanup path preserves inputs and uncommitted outputs and
  releases exactly its initialized descendants, never uninitialized siblings.

## Tests and proof

- Required/optional/nested/active-union positives; missing children, wrong arm,
  shallow copy, duplicate edge, owning cycle and premature commit controls.
- Failure at each finite child construction point, skipped/duplicate cleanup,
  original allocator and unchanged input/output sentinels; private proof tests.
- Full cached focused/tracked/release/lint/conformance gates and uncapped review.

## Commit gate

Record evidence and commit/push; dynamic live families remain required in 03D.
