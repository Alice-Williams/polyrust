# M34A-11-02D — C ownership, bounds and arithmetic proof

- Status: planned
- Depends on: M34A-11-02C

## Goal

Implement this bounded part of M34A-11-02 without introducing a raw-source path
or advertising capabilities before their mappings exist.

## Definition of done

- Derive Empty/Live/Moved/Dropped ownership state, allocator provenance, initialized prefixes, read-only borrow lifetimes and cleanup obligations.
- Authenticate invocation allocator forwarding across every nested call/helper;
  selecting a compatible but different descriptor cannot satisfy this proof.
- Derive range/bounds/tag facts from dominating AST tests and verified operations; never trust a caller-supplied safe flag or nominal proof ID.
- Derive generated callable summaries from bodies and validate the sealed
  known contracts. Authenticate indirect function/member provenance before
  applying ownership, aliasing, failure or allocator transitions.
- Derive/check summaries for legal synthesized lifecycle specialization
  components without assuming a topological body order. Validate the actual
  recursive owned-subvalue traversal and cleanup obligations; a registered
  summary/contract name is not an induction proof. Checked portable v0 still
  rejects direct/mutual user-call recursion; this does not add that feature.
- Derive and enforce the conservative call-sequencing contract in the grammar:
  call-free operands/conditions, full-expression call roots, and branch-local
  short-circuit/conditional prefixes. No caller-supplied purity exemption.
- Reject unsafe signed arithmetic/conversions, zero division, invalid shifts, premature pointer formation, inactive union reads and inconsistent branch/loop cleanup.
- Expose only the private verified C AST state required by the later linker; rendering/capacity certification remains stage 04.

## Tests and proof

- Use-after-move/drop, double ownership, borrow escape, wrong allocator, partial construction, early-return/cleanup-jump and loop/branch join controls.
- Reject nested null/default/source-allocator or different-context substitution;
  clone A from B uses A for new storage while move/drop retain old provenance.
- Signed extrema, checked allocation products, bound-before-pointer and active-tag dominance tests, with mutations that retain labels but invalidate their underlying evidence.
- Allocation restore preserves Uninitialized/Prefix(n): initialization writes
  are legal, reads before initialized dominance are rejected, and Live requires
  complete construction. Test absent/partial/zero initialization separately.
- Same-prototype wrong-contract/callback/member substitutions are rejected;
  an identity registration alone never proves its declared effect summary.
- Authenticate internal PointerTest use in output-slot, allocator-callback,
  allocation-result, interface-context null guards and exact self-move guards;
  reject a portable Equality plan substituted with handle/slot identity.
- Reject nested/sibling/argument calls and hidden condition calls; retain
  controls for aggregate initializers and indexed/member/dereferenced places;
  positive ordered temporaries and conditional prefixes. Mapping-level tests
  independently reject reordered children and eagerly hoisted RHS effects.
- Full focused Rust/compile-fail/lint and cached tracked/release/eight-target gates. Native compiler/sanitizer proof is still required at stages 04/06.

## Commit gate

Record exact commands, invocation IDs and outcomes. Commit and push this slice
with M34A-11-02D; keep its parent M34A-11-02 and overall C compliance open
until their remaining obligations pass. Use focused modules below the source
size limits and distinct Bazel targets only at real independent boundaries.
