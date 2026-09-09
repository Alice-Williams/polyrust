# M34A-11-02D-04B — Storage, initialization and pointer paths

- Status: planned
- Depends on: M34A-11-02D-04A

## Goal

Derive actual storage/subobject identity, initialization and pointer provenance
on the immutable control graph, composing the numeric extent observations.

## Definition of done

- Use shared authenticated Local/Parameter/Global roots and member/index paths,
  plus actual allocation identities; no names, free integers or safety labels.
- Track absent/uninitialized, initialized scalar/field, initialized array prefix
  and complete storage separately. Union active member is a separate fact.
- Derive addresses, pointer copies and qualification/void conversions without
  gaining extent, alignment, ownership or lifetime. Unknown pointers reject.
- Prove bounds and live provenance before pointer formation/access; a write may
  initialize fresh storage but a read requires the selected initialized bytes
  and active member. Do not mistake aggregate zero for a live owner.
- Joins keep only common established facts; loops converge conservatively;
  direct/indirect writes, alias effects and all crossed lexical exits invalidate
  affected facts. No caller-authored initial state or mutable proof escapes.

## Tests and proof

- Field and array path substitution, read before write, partial/complete/zero
  initialization, inactive union member and nested dimension controls.
- Address-only bounds, null versus nonnull, wrong extent/alignment, expired
  stack address, alias write, branch/loop join and cleanup-jump bypass controls.
- Positive useful reads through derived in-bounds pointers, not blanket rejection.
- Private-evidence compile-fail tests, complete cached focused/tracked/release/
  conformance gates and independent uncapped review.

## Commit gate

Commit/push 04B with exact evidence; dynamic allocation/lifecycle remains 04C.
