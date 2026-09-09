# M34A-11-02D-04C-03C-04 — Clone, rollback and finite child closure

- Status: planned
- Depends on: M34A-11-02D-04C-03C-03

## Goal

Prove the complete finite child contract under construction failures and clones.

## Definition of done

- Actual clone builds disjoint allocation graphs; borrowed inputs remain live
  and unchanged when results are independently destroyed.
- Every finite allocation-failure/early/cleanup path releases exactly initialized
  descendants through original provenance and preserves output sentinels.
- Reconcile all parent 03C obligations with composed tests, including role/
  active payload/graph/transfer/storage/numeric/cleanup interaction.
- No unresolved core review finding; no native rendering or dynamic family
  claim is inferred from finite AST proof.

## Tests and proof

- Failure at every finite child allocation point, nested partial rollback,
  skipped/duplicate cleanup, unchanged input/output and independent clone tests.
- Combined stale-storage/valid-owner and valid-storage/invalid-owner controls.
- Full focused/tracked/release/lint/deterministic eight-target gates and fresh
  uncapped review; explicitly distinguish legacy C conformance from typed C.

## Commit gate

Record evidence, close this checkpoint and parent 03C only when every obligation
passes, commit and push. Continue mandatory dynamic owner families in 03D.
