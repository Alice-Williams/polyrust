# M34A-11-02D-04B — Storage, initialization and pointer paths

- Status: complete
- Depends on: M34A-11-02D-04A

## Goal

Derive actual storage/subobject identity, initialization and pointer provenance
on the immutable control graph, composing the numeric extent observations.

## Definition of done

- Use shared authenticated Local/Parameter/Global roots and member/index paths,
  with allocation identities introduced only by the producing-call proof in
  04C; no names, free integers or safety labels.
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

## Implementation sequence and admitted boundary

1. Extract the authenticated root/member/index identity into a shared private
   ownership module. Numeric keys retain exact indices; storage paths may carry
   a checked interval for an actual index occurrence.
2. Add compressed storage values: uninitialized, initialized unknown, zero,
   pointer provenance, record fields, sparse arrays and active union payloads.
   Array defaults and sparse overrides prove prefixes/complete coverage without
   enumerating arbitrary array lengths. Unknown-index writes are weak updates.
3. Derive addresses and copies from actual places. A pointer into an array may
   range only within that same array; a member pointer does not acquire the
   containing record/array extent. Qualification and void conversions retain
   the original typed storage, and restoration must not invent provenance.
4. Run a conservative graph fixed point, then check every reachable action
   strictly. Compose borrowed numeric observations, not reconstructed index
   spellings. Expire crossed-scope storage and invalidate all surviving copies
   of its pointers, including at a later activation of the same declaration.
5. Validate static address initializers separately. Function entries cannot
   assume mutable globals still hold their initializer values. Reject automatic
   addresses escaping through returns or global stores, including aggregate
   copies. Calls/allocation restoration remain unproved until 04C/05 provide
   actual effect/producing-call evidence; a declared contract is insufficient.

`check_storage_paths` is a diagnostic-only intermediate boundary. It does not
replace the contextual initialization prerequisite or certify owning handles,
allocator behavior, boundary validation, linking or rendering. The existing
contextual check remains conservative for direct variable-index automatic
reads; this checkpoint adds pointer/subobject proofs rather than weakening it.

## Required evidence matrix

- Field and array path substitution, read before write, partial/complete/zero
  initialization, inactive union member and nested dimension controls.
- Address-only bounds, null versus nonnull, wrong extent/alignment, expired
  stack address, alias write, branch/loop join and cleanup-jump bypass controls.
- Positive useful reads through derived in-bounds pointers, not blanket rejection.
- Private-evidence compile-fail tests, complete cached focused/tracked/release/
  conformance gates and independent uncapped review.

## Commit gate

Commit/push 04B with exact evidence; dynamic allocation/lifecycle remains 04C.

## Implementation and evidence

Shared authenticated declaration roots and member/index selectors now serve
numeric and storage analysis. The storage consumer borrows actual numeric
occurrences, derives compressed zero/sparse/field/union representations, and
checks provenance, initialization, bounds and lexical lifetime on converged
actual control-flow paths. Exact writes establish initialization; ambiguous
writes and joins lose facts conservatively. Pointer copies retain their original
subobject identity across aggregate copies, aliases, exits and reactivation.

All 374 C unit tests pass. Storage suites cover partial and zero initialization,
same-typed wrong-field substitution, active union copies/joins, nested array
dimensions, scalar/member/array pointer extents, sparse interval reads/writes,
qualification and ABI-compatible type spellings, known null guards, expired
copies, cleanup exits, repeated activation, nested automatic-address escape,
actual adapter restoration, and unproved call/allocation/imported-pointer
boundaries. New production modules remain below the file-size limits.

Passing cached Linux Dev Container evidence:

- Five focused targets (C units, Rustdoc/compile-fail, Clippy, Buildifier, docs):
  `f8936d4c-312c-4beb-93b1-801e4c4fa8ba`.
- Full tracked graph: `4c0909fb-ba09-4e85-a2bf-dd8d0b85864b`
  (445 rules, 320 test targets pass, 45 executed).
- Release: `6deda5bf-640c-4884-92a5-aecc5975ab57`
  (257 test targets pass).
- Deterministic eight-target conformance:
  `f454fa8e-b0bb-461c-9544-b98f83485636`
  (50 cases plus one portable test agree; repeated manifests are byte-identical).

Independent uncapped Sol Extra High review, c17_contextual_final_review,
identified three defects. All were accepted and repaired; none was dismissed
as an optional feature:

1. Storage target matching rejected AST-admitted Int/I32 and U64/Size spellings.
   AST qualification conversion and storage/adapter target matching now share
   the authenticated registry relation, retaining exact nominal identities,
   array bounds and nested pointer qualifiers.
2. Zero and explicit-null representations lost their common pointer fact at a
   join. Joins now normalize only equal proved pointer facts, recursively through
   aggregates, without promoting uninitialized, unknown or expired values.
3. Known standard-stream pointers bypassed the strict imported-pointer boundary
   when discarded. Every evaluated expression result now satisfies the same
   complete/provenance invariant, also covering conditional and interval joins.

The three regressions ran red together in
`145ebe18-b793-44e7-8e2f-dcfb70fc626e` (366/369 units passed). The first two also
had individually recorded red controls. A different existing Sol Extra High
reviewer, c17_contextual_repair_review, subsequently audited the complete scoped
implementation and repairs with no substantive defect or required proof gap.
These reused reviewer contexts are not represented as fresh blind reviews.
Code/spec/tests were frozen during that final audit; all final gates above used
the same production/test tree. Review-requested field substitution, imported
and ambiguous pointer-only reads, and recursive null joins are directly tested.

Development failures are not pass evidence: initial Rust privacy, enum-size,
index-type and fixture API/inventory errors were corrected before the gates.
The late imported-parameter fixture failed contextual inventory (373/374 units,
`2a820e74-00a1-4631-8c93-a2602917dd4d`); retaining its other registered function
definition corrected the fixture without changing production checks.

This closes 04B only. The parent 04, allocation/lifecycle 04C, boundary/borrow
04D and body-summary 05 contracts remain open. Calls and allocation restoration
still reject; this diagnostic boundary grants no owning-handle or rendering
certificate. Existing legacy C conformance is regression evidence, not typed-C
cutover evidence. The user-owned stdlib-abs tree remains untouched.
