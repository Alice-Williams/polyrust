# Multiple closed owned producer chains

- Status: implemented compiler-only proof; existing one-chain APIs remain restricted
- Plan: [M35-02B-03B](../../../../plan/tasks/M35-02B-03B-multiple-owned-chains.md)
- Foundation: [tail scopes](rust-owned-tail-scopes.md)

## Initial source shape

Use safe non-generic Rust functions with immutable i32 parameters, i32 result,
and the existing tail-nested block structure. Each plain immutable let is either
an authenticated Box<i32>::new of a directly resolved parameter or a whole-value
move from an already declared owned binding. The final expression dereferences
one remaining live binding. No arithmetic, borrowed aliases, branches, explicit
returns, sibling blocks, projected moves or other calls are admitted here.

Use budgets of 128 parameters, 128 owned bindings and 64 nested scopes; these are
experiment resource limits, not a cap on the generic AST's function arity.

Every constructor must use a distinct parameter binding. A repeated argument
binding diagnoses ambiguity for this increment, even if both constructors are
individually genuine. Do not use allocation counts, equal payload types, debug
entries, spans or incidental traversal positions to distinguish them.

## Authenticated relation

Match canonical HIR parameter positions to the compiler's MIR argument locals.
Trace each authentic MIR constructor argument through uniquely defined scalar
copy/move producers to that actual parameter. The distinct anchor identifies
one source construction; its result place starts that source owner's chain.

Associate each whole-value move with its HIR-resolved source binding and the
matching typed MIR source/destination edge. Require complete, disjoint chains:
all Box locals and relevant assignments/calls must be accounted for, no owner
may be shared by chains, and no extra definition, reassignment or call is ignored.

Use the actual final dereference's producer to select its owner. Every remaining
chain end must have exactly one drop after the scalar return read on the complete
normal path. Map each owner to the scope of its final live binding, preserving
canonical block nesting. Check cleanup order against reverse declaration order
of the final live bindings on this tail-only path; moved bindings have no drop.
No compiler drop flag or branch is interpreted in this increment. Every Drop
must have no async-drop cleanup successor (`drop: None`); an extra successor
rejects even when it points to an already inventoried normal-path block.

Local-number bijections are not corruptions: consistently renaming a MIR owner
local in its constructor destination, all uses/moves/drops, storage markers and
declaration table preserves the relation. Its HIR binding maps to the renamed
place because it still has the same authenticated parameter-derived producer.
We do not freeze pre-renaming numeric labels or consult stale debug metadata as
an independent binding authority. Partial substitutions that change producer,
read or cleanup relations must reject. The test suite checks both cases.

Pinned PostCleanup MIR can retain write-only Boolean bookkeeping after its
conditional cleanup edges have disappeared. Account for such residual writes
only when every remaining assignment is a Boolean constant to an unprojected
Boolean local and a complete MIR local-use visitor finds no use other than those
exact assignment destinations and storage markers. Any read, address-taking,
nonconstant definition, branch use or unmatched operation diagnoses. This proves
the residual values are unobserved on the admitted body; it does not interpret
their values as ownership/drop-flag evidence or admit conditional cleanup.
Use the compiler's [MIR visitor](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/mir/visit/trait.Visitor.html)
and closed place-use contexts; pinned compilation and mutation tests, not moving
nightly documentation, establish the implemented API and behavior.

## Evidence and exclusions

Expose only private, compiler-session evidence containing constructor/parameter
anchors, source bindings and places, move locations, final read and ordered drop
obligations with their lexical scopes. Reuse existing compiler identities and
capability contracts; do not introduce a parallel untyped ownership AST.

Repeated constructor anchors, non-parameter initializers, custom destructors,
partial moves, branches and call-boundary transfers remain unsupported. This
relation does not select C allocators, emit cleanup, replace the borrow checker,
or certify target behavior. Native allocation/cleanup proof remains M35-02C/D.

## Implementation and proof targets

The separate `MultipleOwnedBody::read` entry retains private per-chain
constructor, parameter, source/MIR binding, move, drop and scope evidence. The
root-only and one-chain tail-scope entries keep their original restrictions.
Shared `Trace` accounts for the complete finite path; the old `Flow` wrapper
still requires exactly one call and one drop. Canonical containment facts are
shared without giving multi-owner bodies a misleading single drop scope.

`owned_multiple_test` covers nine positive and five rejected valid-Rust
functions, including parameter order differing from declaration order, equal
types, interleaved moves, nested/shadowed bindings, unread owners and different
return owners. Twenty-one private body corruptions and an isolated residual-use
visitor corruption must reject; substituting a genuinely different unread
Boolean constant and a bijective owner-local renaming must still pass. The
harness requires both mutation and renaming markers,
two valid-Rust fixture mutations, an empty-inventory failure and compiler
E0382/E0502 rejection before any success marker.

`owned_multiple_private_body_test` and `owned_multiple_private_chain_test`
each require exactly one E0451, without accepting incidental diagnostics.
`owned_multiple_format_test` covers all nested source modules, tests and fixture;
compiler-adapter builds run Clippy with warnings denied. Existing constructor,
linear and scope proofs remain enabled. Test-only MIR mutation entry points are
not part of the evidence constructor API.
