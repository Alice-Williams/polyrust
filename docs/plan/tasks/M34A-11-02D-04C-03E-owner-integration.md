# M34A-11-02D-04C-03E — Composed owner and parent audit

- Status: planned
- Depends on: M34A-11-02D-04C-03D

## Goal

Close owner 03 and allocation/lifecycle 04C only with complete composed evidence.

## Definition of done

- Reconcile every parent obligation against actual implementation and tests;
  local leaf, child, prefix/family, allocator and exit proofs compose without
  trusting registered contracts or silently ignoring unsupported effects.
- Exercise mixed valid storage/invalid ownership and valid ownership/stale
  storage cases, including numeric losses and outstanding branch resources.
- Record remaining incoming-ABI 04D, generated-summary 05 and native-runtime 06
  boundaries without moving any current parent requirement into those stages.
- Independent uncapped review has no unresolved core finding; assess all
  feedback explicitly and regress accepted repairs.

## Tests and proof

- Complete parent positive/negative matrices, combined sentinel/cleanup/alias/
  allocator mutations, private evidence controls and source-size/Bazel wiring.
- Full cached focused/tracked/release/lint/deterministic eight-target gates;
  do not mislabel legacy C conformance as new C rendering proof.

## Commit gate

Record exact evidence, close parents 03/04C, commit and push; continue 04D.
