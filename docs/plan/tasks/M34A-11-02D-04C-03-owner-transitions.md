# M34A-11-02D-04C-03 — Construction, transfer and cleanup transitions

- Status: in-progress
- Depends on: M34A-11-02D-04C-02

## Goal

Complete 04C's owner lifecycle proof independently of storage initialization.

## Ordered checkpoints

The [owner-transition specification](../../specification/typed-generation/languages/c/owner-transition-proof.md)
defines the contract/evidence boundary. Implement in this order:

1. [03A: typed owner-slot contracts](M34A-11-02D-04C-03A-owner-contracts.md).
2. [03B: local construction, move and drop](M34A-11-02D-04C-03B-local-owners.md).
3. [03C: owned children and rollback](M34A-11-02D-04C-03C-owned-children.md).
4. [03D: dynamic owner families](M34A-11-02D-04C-03D-owner-families.md).
5. [03E: composed owner audit](M34A-11-02D-04C-03E-owner-integration.md).

Each completed checkpoint requires its own cached gates, independent uncapped
review, commit and push. Contracts alone keep owner admission rejected. Local
leaf proof cannot close this parent; all child/family obligations below remain
required. Incoming ABI/custom allocator contracts stay 04D and actual generated
body summaries stay 05, as already specified, without new deferrals.

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
