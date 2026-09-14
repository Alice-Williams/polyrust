# Tail-nested owned scopes

- Status: implemented compiler-only proof; separate admission preserves root-only behavior
- Plan: [M35-02B-03A](../../../../plan/tasks/M35-02B-03A-tail-scopes.md)
- Foundation: [linear correspondence](rust-linear-owned-correspondence.md)

## Source contract

Retain the foundation's safe non-generic one-i32-parameter/i32-result function,
one authenticated Box<i32> construction and immutable whole-value move chain.
Permit an unlabeled block as a block's tail expression, recursively. The eventual
tail must dereference the latest binding. The constructor may appear in any
block, provided preceding wrapper blocks contain no other statements.

For example, the inner binding owns the allocation at the final read and exit:

```rust
pub fn value(input: i32) -> i32 {
    let outer = Box::new(input);
    {
        let inner = outer;
        *inner
    }
}
```

The source scope tree is a chain in this initial increment. It is still a tree
of canonical HIR blocks, not a flat sequence with inferred anonymous scopes.
For each block retain its HirId and optional canonical parent, for each binding
its declaration scope, and for the final dereference its containing scope.
Keep the existing 128-binding budget and limit scope depth to 64; these are
resource budgets on the experiment, not general language limits.

## Correspondence and cleanup

There is exactly one construction/move producer chain across the nested blocks.
Authenticate the compiler places and operations using the foundation's complete
typed relation. Scope membership comes from canonical HIR containment, never
from a debug scope or a source-span overlap. Before attaching scope metadata,
verify all members/parents against that canonical tree, including empty wrappers.

The final scalar read and single normal-exit drop belong to the scope containing
the final live binding. Tail wrapper scopes below that declaration can contain
the read without introducing a new owner; retain read scope and owner/drop scope
separately. Moving ownership into a child scope does not leave a second cleanup
obligation in its parent. Enclosing scopes return the already computed scalar.

The compiler's single drop after the read and before return remains mandatory.
This limited shape does not infer cleanup placement for sibling blocks or
multiple normal exits. Such forms require a separate correspondence contract.

## Boundary and proof

Continue excluding additional calls/owners, mutable bindings, projections,
branches, explicit returns, non-tail blocks, partial moves and function-boundary
transfers. Reject extra or unaccounted MIR operations. Keep existing no-heap
backend publication controls enabled. No C layout, allocator or generated
cleanup policy is selected here.

Positive tests distinguish root scope, binding scope and read scope even when
all names are identical. Negative tests substitute scope membership and parent
edges, omit/duplicate scopes, and add cleanup for moved outer bindings. Actual
compiler MIR remains the only non-test input to the evidence constructor.

## Implementation boundary and proof targets

`LinearOwnedBody::read` keeps the root-only contract; `read_tail_scopes` selects
the tail-block extension explicitly. Both use the same source-operation checks
and complete MIR producer/move/read/drop relation. `scopes::certify` independently
walks the canonical HIR block chain to check proposed parent edges, binding
membership, read scope and the final live binding's declaration/drop scope.
Only the private `ScopeEvidence` carries accepted claims; it retains canonical
block references and exposes read-only identities. It is not target admission.

`owned_scope_test` covers seven positive shapes, five valid-Rust exclusions,
ten corrupted scope claims, a wrong-body claim and an extra moved-outer-owner
cleanup. Its source controls mutate an otherwise valid fixture's wrapper/return
and reject empty inventories; E0382/E0502 fail before success/mutation markers.
`owned_scope_private_test` requires exactly one E0451 when constructing fake
ScopeEvidence. `owned_scope_format_test` covers the fixture and every probe/
production module; compiler builds run Clippy with warnings denied. The three
existing `owned_linear_*` tests remain required and unchanged in purpose.

The observed single-drop relation establishes owner cleanup only for this
tail-only normal path. It does not authenticate arbitrary multi-exit cleanup,
model all MIR storage events or claim that lexical scope names alone prove
runtime destruction. Backend allocation/cleanup and native equivalence remain
later M35-02C/D work.
