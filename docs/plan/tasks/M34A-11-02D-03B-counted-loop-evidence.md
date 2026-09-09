# M34A-11-02D-03B — counted loop evidence

- Status: planned
- Depends on: M34A-11-02D-03A

## Goal

Actual-AST counted-loop structural and path evidence; facts name the exact loop/declarations/steps, not caller metadata.

## Definition of done

- Actual counter Size(0) declaration and immutable Size bound snapshot dominate entry; bound need not be a literal.
- Exact Numeric(Bool,Less(Read(counter),Read(bound))) and actual Add Size(1) assignment nodes.
- Inventory all writes/addresses and reject foreign/nested mutation, counter sharing/substitution and writable bound aliases.
- Path-state counts exactly one step on every continuing path; early exits need no step; nested loops and switches retain exact targets.
- Private evidence consumed later for interval refinement, not whole numeric or lifetime certificate.

The detailed numeric contract is in
[numeric range proof](../../specification/typed-generation/languages/c/numeric-range-proof.md).
The exact loop form remains [counted loops](../../specification/typed-generation/languages/c/counted-loops.md).
These slices implement the parent task; none weakens its complete-stage gate.

## Tests and proof

- Complete counted-loops.md mutation matrix with zero/one/multiple/SizeMax boundary controls.
- Branch-local one-step positive vs missing/double steps and Continue bypasses.
- Nested-loop/switch exits, declaration reentry/intervening writes, alias/escape and owner substitution.
- Full cached gates and independent review.

## Commit gate

Record focused and full cached Linux Dev Container evidence and an independent
uncapped Sol Extra High review. Commit and push M34A-11-02D-03B separately. Keep
parent 02D-03 open until every slice and its combined checklist pass. No partial
numeric or loop fact is a C safety, allocation or rendering certificate.
