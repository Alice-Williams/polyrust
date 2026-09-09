# M34A-11-02D-04C-03A — Typed local owner contracts

- Status: complete
- Depends on: M34A-11-02D-04C-02

## Goal

Give ownership obligations typed declaration origins before implementing flow.

## Definition of done

- COwnerSlotRef privately retains its existing authenticated local; registration
  validates the exact mutable object-pointer category and preserves source type.
- Duplicate, cross-registry, const/borrow, callback, void, stream and wrong-type
  bindings reject. Nominal forward registration does not claim complete layout.
- Inventory and package revalidation include the role and actual local occurrence;
  no alternate declaration, mutable proof map or string ownership identity.
- Storage safety explicitly rejects registered owner obligations until 03B
  integrates actual transitions. No registration creates private live evidence.

## Tests and proof

- Constructor/category/alias/foreign/missing-occurrence and private reconstruction
  controls; positive ordinary mutable object-pointer and nominal registrations.
- Compile-fail field-construction/mutation tests and a real AST showing that an
  empty owner registration alone cannot make storage admission succeed.
- Focused C/compile-fail/Clippy/docs/Buildifier plus full cached tracked, release
  and deterministic eight-target gates; uncapped independent review.

## Commit gate

Record exact evidence, commit/push this contract checkpoint, then continue 03B.
Neither this checkpoint nor its rejected owner boundary closes parent 03.

## Implementation checkpoint

COwnerSlotRef has only its private existing local reference. Registration and
rechecking authenticate exact registry/function membership, mutable slot and
pointee category, including aliases and array qualification. Canonical inventory,
actual occurrence, origin-role and object-form checks consume the new closed
registration variant. The storage boundary returns UnprovedOwnership for these
roles until the next lifecycle checkpoint; no live/transfer claim is exposed.

Focused invocation e4132bb3-b417-475b-9c69-f75fd1cd5345 passes all 524 C
units, typed compile-fail, Clippy, documentation and Buildifier. Nine new unit
tests cover identity/inventory/freeze, categories and aliases, actual occurrence,
registration-only rejection, incomplete nominal and fixed-array roles, foreign/
wrong-function/duplicate bindings, deterministic inventory, and private wrapper/
inventory reconstruction. Two compile-fail examples reject field construction
and retargeting.

Tracked invocation 8f027a05-f03a-4e94-949a-018452d8b9aa passes all 320
test targets (445 rules including non-test targets). Release invocation
05b506aa-3200-4165-bb57-347fc8af50eb passes all 257 tests. Conformance
611ec682-f547-4b24-8faa-a4ce18469e73 passes 50 cases and one portable
test across eight targets, with byte-identical repeated manifests. C conformance
still uses the legacy backend; this is not typed-C rendering evidence.

Independent Sol Extra High reviewer c17_owner_contract_review inspected the
complete frozen source/test/specification/plan delta and supporting integration,
with no finding limit: PASS, no substantiated core defect or required proof gap.
The maintainer accepts that assessment. The optional combined incomplete-nominal
package test is not a blocker: registration coverage and traced package/layout
checks establish the current contract without claiming complete pointee layout.
Local transitions, children, families and composed closure remain 03B through 03E;
parent 03 and the C migration remain in progress.
