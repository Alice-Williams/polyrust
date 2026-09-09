# M34A-11-02D-04D — Borrows, allocator flow and boundary composition

- Status: planned
- Depends on: M34A-11-02D-04C

## Goal

Complete the parent storage contract, including read-only borrow lifetimes,
invocation allocator identity and the ordered public-boundary obligations.

## Definition of done

- Bind borrows to actual owners, extents and scope; reject owner mutation/drop,
  escaping views and ownership manufactured by pointer copies.
- Derive null/default/custom allocator selection from actual AST; retain the
  invocation descriptor including context, not just matching callback types.
- Authenticate exact allocator forwarding obligations at nested call sites.
  Null/default/source-allocator/foreign-context substitutions fail. Clone A
  from B keeps new storage A and existing storage B; move/drop preserve origin.
- Preserve callable-abi.md validation order, rejected output contents and
  allocation-free drop obligations. Complete all parent 04 tests and integrate
  numeric/storage facts over the same immutable package.
- Do not bootstrap generated call effects from prototypes or labels: unresolved
  body contracts remain explicit 05 obligations. Their later discharge cannot
  weaken local allocator/borrow/boundary proofs or grant a renderer bypass.

## Tests and proof

- Borrow escape, mutation/drop while borrowed, same-object read-only alias
  positives, wrong nested descriptor/context and clone A-from-B controls.
- Pairwise applicable boundary failures, unchanged sentinels, failure cleanup,
  null/default/custom selection, branch/loop/crossed-scope combinations.
- Full parent checklist, private proof compile-fail boundaries, complete cached
  focused/tracked/release/conformance gates and uncapped independent review.

## Commit gate

Close 04D and parent 04 only with all required local evidence; commit/push and
continue 05 body-summary discharge. C compliance remains incomplete.
