# M34A-11-02D-04C — Allocation and lifecycle transitions

- Status: planned
- Depends on: M34A-11-02D-04B

## Goal

Authenticate actual allocation results and partial construction, move, release
and cleanup transitions rather than trusting registered ownership labels.

## Definition of done

- Bind allocation restore to the actual successful allocation result, checked
  byte extent, measured alignment, concrete type and exact allocator origin.
  Registration or cast alone cannot turn uninitialized storage into Live.
- Track Empty/Live/Moved/Dropped independently from construction progress.
  Actual writes establish initialized fields/prefixes; commit needs completion.
- Preserve owners on failure; release exactly once with the allocating
  descriptor. Reject shallow owning copies, double release and use after move.
- Check every normal/error/cleanup exit and crossed scope; partial rollback
  touches only established initialized storage. Unknown generated effects
  remain diagnostics pending body summaries in 05.

## Tests and proof

- Null allocation, wrong restored type/extent/alignment/allocator, forged
  registration, alias restoration and reused/expired allocation-result controls.
- Partial construction, early return, cleanup jumps, conditional commit, repeated
  drop/move, self-move and branch/loop joins; preserve outputs on rejection.
- Useful complete construction/cleanup positives and exact failure controls.
- Complete cached focused/tracked/release/conformance gates and independent
  uncapped review. Native generated fault-injection proof remains stage 06.

## Commit gate

Commit/push 04C with exact evidence; parent 04 remains open for 04D composition.
