# M34A-11-02D-04C-03B — Local construction, move and drop

- Status: complete
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

## Implementation grammar

Keep private owner states in the existing paired storage/numeric product. Prepare
transitions against the incoming snapshot; apply them only after both ordinary
transfers succeed. Outstanding or uncertain states survive fallback and joins.
Owner writes must target the actual direct local, not an alias to its slot.
Claim only an already bound, completely initialized fixed allocation base whose
entire payload type contains no pointers. Borrowed construction temporaries do
not become additional owners. Move copies read the actual source owner local.
The next graph action must reset that source to null, through a single flow edge
with no crossed scope or alternate predecessor. Release recognizes the closed
default release call with a direct owner read and object-to-void erasure; a live
owner cannot instead be freed through a raw alias. A live release similarly
requires the immediate null reset. Empty release is harmless without a reset.

At completed move, preserve the physical allocation and numeric contents, retire
all prior pointers into it, and restore only the destination owner pointer. Drop
uses existing physical release retirement. Validate every exited scope, return,
function end and redeclaration before discarding settled local roles. Unknown
or unequal live joins cannot manufacture an empty slot or completed transaction.
Revalidate live slot/base identity and active-payload completeness after each
successful paired action, including mutations made through borrowed aliases.

## Commit gate

Record evidence and commit/push; continue children 03C without closing parent 03.

## Implementation and proof checkpoint

Private owners/actions/claims/transactions modules compose with the existing
numeric/storage product; no source fragments, public evidence constructors or
new dependencies are introduced. Boxed enum variants retain typed local and
allocation origins. Actual declaration/assignment/call nodes establish empty,
live, move-reset and release-reset states. Provisional failures preserve owner
obligations, joins cannot settle uncertainty, and crossed scopes/returns reject
unaccounted owners. Move retirement preserves the same physical allocation and
numeric history while expiring every old pointer copy.

Focused invocation 179b7f95-965f-4e57-82a0-6868f143be99 passes all 549 C
unit tests, typed compile-fail, Clippy, documentation and Buildifier. New coverage
includes scalar/record/union/fixed-array construction and moves, original base
versus interior/automatic storage, shallow aliases and repeated moves, raw alias
release, adjacent reset grammar, slot-address writes, branches/cleanup/scopes,
loop reactivation, exact/wrapped numeric history, private joins and retirement.
The prior registration-only empty-slot rejection is replaced by proof of the
actual null initializer; registration still supplies no lifecycle evidence.

Maintainer audit found that an active-union-arm write could invalidate an
already-live owner's complete payload without removing its transfer authority.
Actual AST regression 62910436-f6cd-483a-b9b7-22c88f56a2ec reproduced wrong
acceptance before repair. Every successful paired action now revalidates live
slot/base and complete active payload, with only the exact pending release-reset
slot exempt. Focused 92035c4c-2e85-4490-97cd-5c65ba1928c3 passed the repair.

Mutation invocation 1855aade-aa5f-4d4e-a8dd-64e311b385d7 temporarily removed
move-time alias retirement: the actual stale-borrow regression failed with wrong
acceptance. The original implementation was restored, and the final focused
gate above passes. No mutation remains. Strict replay may diagnose the concrete
expired/uninitialized storage operation or a poisoned owner join first; negative
tests also have valid controls and contextual package checks where appropriate.

The first independent uncapped review found one contract defect: a pending
copy/reset or release/reset could enter a descendant block because the flow edge
only recorded exited scopes. Both actual-AST regressions reproduced wrong
acceptance in invocation 0112c2b3-b6fe-443e-b446-5f024aedd893, with same-scope
positive controls. Pending transactions now require identical adjacent node
scopes as well as the existing single-flow/no-alternate-predecessor rules.

Refreshed focused invocation 9761ac5b-c52a-496c-935d-57360ffc9df4 passes
all 551 C units, typed compile-fail, Clippy, documentation and Buildifier.
Tracked invocation 127caa5d-4759-42da-8932-f6f76bbc65e9 passes all 320
test targets; release 37895878-0294-4ed2-8013-50936dd59d61 passes all 257.
Conformance a874b02c-5e2c-406f-9cc8-698ff7e03146 passes 50 cases and one
portable test across the evaluator and eight targets, with byte-identical
repeated manifests. C conformance still uses the legacy generation path.

Fresh independent Sol Extra High reviewer c17_local_owner_final_review audited
the entire repaired source/test/spec delta and supporting integration without
a finding limit: FINAL PASS, no findings. The maintainer accepts the assessment;
the previous scope-entry finding was agreed, reproduced and repaired rather
than waived. Review covered claims/provenance, complete live payloads, actual
transaction grammar, alias/numeric preservation, joins/fallback, all exits,
reactivation, privacy and Bazel wiring. The reviewer did not rerun builds; the
maintainer ran the exact refreshed gates above. This closes local leaf proof,
not child/family ownership, custom/imported effects or typed-C rendering.
