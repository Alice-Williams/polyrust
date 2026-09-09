# M34A-11-02D-04C-03C-01 — Typed child member contracts

- Status: complete
- Depends on: M34A-11-02D-04C-03B

## Goal

Register typed required/optional child and borrowed metadata obligations over
existing member declarations without manufacturing lifecycle evidence.

## Definition of done

- Private CMemberOwnershipRef retains exact member/type/nominal origin and a
  closed CMemberOwnership enum; one authoritative role per existing member.
- Shape and alias checks implement the child-ownership specification, including
  fixed-array terminals, const distinctions and forward nominal registration.
- Canonical inventory distinguishes role categories. Package/type/origin checks
  require the actual member occurrence and revalidate role shape/membership.
- Child-role storage admission explicitly rejects until graph integration;
  no-role programs and the proved local-leaf path remain unchanged.
- Focused production modules, no new dependencies or raw source/evidence flags.

## Tests and proof

- Required/optional/metadata/fixed-array/alias/forward-nominal positives.
- Duplicate/conflicting/foreign/wrong-owner/category/const/void/callback/FILE
  negatives; inventory remains unchanged on failed registration.
- Actual package with/without member definition; role-only safety rejection
  paired with the same no-role program.
- Private wrapper/map/role reconstruction and compile-fail construction/mutation.
- Focused C, typed compile-fail, Clippy, docs and Buildifier; full cached tracked,
  release and deterministic conformance gates; fresh uncapped review.

## Commit gate

Record exact evidence, commit and push. Continue 03C-02; parent 03C stays open.

## Implementation checkpoint

CMemberOwnershipRef retains private member/role fields; the authoritative map
allows one role per exact member. Required/Optional share the existing local
owning-pointer shape check. Metadata admits only const object/function pointer
categories, without claiming an actual static lifetime. Canonical inventories
retain distinct role kinds and contextual inventories retain both actual map
key and role value. Package/type/origin passes consume the closed variant.
Storage safety explicitly rejects child obligations pending graph integration.

The first focused gate d7d513c6-b734-4b10-87f0-8b4ec18424e4 passed 563 C
units and companion gates. Maintainer audit then found a private integrity gap:
an extra mismatched map key could hide behind a valid role stored under its
correct key. Actual package regression 51705ecb-b311-4418-9b81-8c296729c0bb
reproduced wrong acceptance with a valid control. The contextual inventory now
retains and verifies each actual key/value pair, not just map values.
Refreshed focused gate 4840626e-9f3d-4757-9132-e0a55b79c325 passes all
564 C units, typed compile-fail, Clippy, documentation and Buildifier. Tracked
c261da20-cf87-470f-b426-f746d5ef506d passes all 320 test targets; release
f2ef51f5-e60c-4797-afaf-b5850511d331 passes all 257. Conformance
aa52c458-4eb3-44c1-bf59-22b487cf3cff passes 50 cases and one portable
test across the evaluator/eight targets with byte-identical repeated manifests.
The C conformance path remains legacy, not typed-C rendering evidence.

Fresh independent Sol Extra High reviewer c17_child_contract_review audited
the complete frozen source/test/spec/plan delta and prerequisite integration
without a finding limit: PASS, no correctness or required proof gap. The
maintainer accepts the assessment. Review included map-key integrity, alias/
const categories, actual occurrences, exhaustive consumers, privacy, unchanged
local-owner helper semantics, Bazel wiring and staged obligations. The reviewer
did not run builds; the maintainer ran the complete gates recorded above and
provided their final results after review. Parent 03C remains in progress.
