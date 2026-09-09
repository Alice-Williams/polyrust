# M34A-11-02D-04C-03 — Construction, transfer and cleanup transitions

- Status: planned
- Depends on: M34A-11-02D-04C-02

## Goal

Complete 04C's owner lifecycle proof independently of storage initialization.

## Definition of done

- Derive Empty/Live/Moved/Dropped and partial construction from actual writes,
  calls and slot resets. A shallow pointer copy is not clone or ownership.
- Require complete initialized children and exact allocator at commit. Move
  transfers exclusive ownership and empties the source; invalid/self moves
  preserve outputs. Repeated public drop is safe through an already empty slot,
  not permission to release the same allocation twice.
- Check every normal/failure/cleanup edge. Retain inputs and uncommitted outputs
  on failure; partial rollback releases only initialized owned descendants.
- Support multiple allocations produced by construction loops through proved
  finite ownership/work invariants, without identifying all dynamic instances
  with one singleton or reviving aliases from earlier activations.
- Compose actual allocator provenance and lifecycle obligations without trusting
  generated prototypes. Unresolved body effects remain 05 diagnostics, and
  incoming custom allocator/borrow validation remains 04D.

## Tests and proof

- Partial construction, conditional commit, skipped cleanup, early returns,
  repeated move/release, self-move, shallow owning copies and allocator swaps.
- Multiple live loop-produced children, exact cleanup counts, failure prefixes,
  stale aliases and unchanged output sentinels, with useful positive programs.
- Complete parent 04C matrix, private evidence tests, focused/full cached gates
  and independent uncapped review. Native fault injection is additionally 06.

## Commit gate

Close and commit/push 04C-03 and parent 04C only when every parent obligation is
covered. Continue 04D; neither diagnostic success nor registration certifies C.
