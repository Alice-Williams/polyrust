# M34A-11-02D-04C-03C — Owned children and partial rollback

- Status: in-progress
- Depends on: M34A-11-02D-04C-03B

## Goal

Compose initialized representation with exclusive, complete owned descendants.

## Ordered implementation

The [child-ownership specification](../../specification/typed-generation/languages/c/child-ownership-proof.md)
defines the exact representation and proof boundary. Preserve every parent
obligation below through four independently gated checkpoints:

1. [03C-01: typed child contracts](M34A-11-02D-04C-03C-01-child-contracts.md).
2. [03C-02: finite ownership graph](M34A-11-02D-04C-03C-02-child-graph.md).
3. [03C-03: subtree transfer and destruction](M34A-11-02D-04C-03C-03-child-transfer.md).
4. [03C-04: clone, rollback and composed closure](M34A-11-02D-04C-03C-04-child-rollback.md).

Registration cannot replace proof. Early checkpoints fail closed on unsupported
transitions and cannot close this parent or advertise typed-C generation.

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
