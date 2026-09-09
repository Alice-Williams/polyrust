# M34A-11-02D-04C-03B — Local construction, move and drop

- Status: planned
- Depends on: M34A-11-02D-04C-03A

## Goal

Derive local leaf-owner transitions from actual numeric/storage graph actions.

## Definition of done

- Private states separate uninitialized/empty/live/moved/dropped and unfinished
  copy-reset/release-reset obligations; uncertain joins retain resources.
- Complete pointer-free fixed payload construction can claim one actual live
  default allocation. Partial/interior/duplicate claims reject.
- Actual copy/reset moves require distinct empty destination and live source;
  actual release/reset drops use the original allocation once. Empty drop repeats
  safely; stale borrowed aliases cannot survive move or drop.
- Preserve numeric history and live transferred contents. All scope/return/
  cleanup/redeclaration paths account for owners; solver fallback fails closed.
- Lift 03A's blanket rejection only for these proved local cases. Children,
  dynamic families and unresolved imported/body effects remain explicit errors.

## Tests and proof

- Valid scalar/fixed aggregate construction, move chains and repeated empty drop.
- Partial payload, shallow copy, missing/wrong reset, intervening action, double
  move/free, self move, allocator/base substitution and stale-view mutations.
- Branch/loop joins, early return, cleanup jump, scope reactivation and private
  transition controls; each negative has a useful positive counterpart.
- Full cached focused/tracked/release/lint/conformance gates and uncapped review.

## Commit gate

Record evidence and commit/push; continue children 03C without closing parent 03.
