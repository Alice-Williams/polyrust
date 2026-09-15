# Complete nested-record source correspondence

- Status: implemented for the closed grammar below; no C/Java heap admission
- Plan: [M35-02B-03L-03](../../../../plan/tasks/M35-02B-03L-03-nested-correspondence.md)
- Operation input: [nested construction](rust-nested-construction.md)
- Compiler observations: [nested owned records](rust-nested-owned-records.md)

## Closed initial source grammar

The first body reader accepts a safe nongeneric Rust free function with exactly
three immutable simple i32 parameters and an i32 result. The canonical body is
one unlabeled root block. Its statements are initialized immutable simple lets,
without let-else, followed by one tail dereference or explicit return of a
dereference of a live local scalar Box. Bindings use actual HirId, so shadowing
does not alias an earlier binding. Reject adjustments and other statement,
initializer, control-flow or signature forms.

The first three lets construct standard Box<i32, Global> values, each from a
different parameter. The fourth constructs Inner, a local nongeneric default-
representation named-field struct of two scalar Boxes. The fifth constructs
Outer, whose declaration-ordered fields are Inner then scalar Box. Each source
initializer is a local ownership transfer, and the three constructors account
for all three final leaves. Either aggregate may reverse initializer order;
the compiler field index determines declaration membership, never spelling.

Subsequent lets may move a live scalar Box local or extract a live leaf/whole
Inner through a typed field path. Whole-Inner movement requires both leaves
still present and relocates them together. Reject a whole Outer transfer in
this initial reader, reconstruction, extra constructors/aggregates, borrowing,
cloning, mutation, calls, conditionals and nested executable blocks. The final
read must select a live extracted local leaf, not an aggregate or a moved owner.

This body grammar is intentionally narrower than L-02's operation metadata.
It has exactly two record levels, four aggregate field occurrences and three
owning leaves. Limit the complete statement list to 128, a source field path
to two projections, MIR locals to 1,024 and normal blocks to 512. The existing
whole-flow inventory enforces the last two limits before traversal. Wider
trees and control flow require explicit future grammar/proof changes; 03M is
the separate required conditional-initialization/partial-move checkpoint.

## Source paths and ownership provenance

A query-only source place contains its canonical local HirId and a sequence
of compiler-resolved nominal fields. Each field retains parent Ty, actual
declaration DefId, FieldIdx and full result Ty. A nested place is not a dotted
name or an invented integer identifier. Resolve every source projection using
typeck and the corresponding actual declaration, requiring an unadjusted base
and exact field type at every level.

Maintain each constructor-rooted leaf's current source place. An aggregate
initializer relocates every leaf under its local source into the destination
field prefix. A later field extraction relocates only the selected subtree;
a local move relocates the complete leaf. Reject stale sources, duplicate
ownership, incomplete subtree movement and paths outside the admitted layout.
Retain canonical source order independently from recursive declaration order.

## Complete normal-MIR relation

Read MIR only from the successful-analysis compiler query for this owner, at
Runtime PostCleanup. The safe body entry point takes no caller-supplied MIR.
Require a complete acyclic normal trace with exactly three authenticated
constructor calls, two aggregate assignments, complete cleanup of three
terminal leaves and one full Return location. An intact Inner is dropped as
one record Place by the pinned compiler; a partially moved Inner instead has
separate drops for its remaining leaves. Represent these as a closed cleanup
kind enum, not three fabricated MIR Drop locations. Abort-only unwind policy
remains pinned.

Authenticate every constructor's actual FnDef/GenericArgsRef and full Box Ty,
its parameter identity and scalar staging chain. Map each source binding to
the unique typed MIR destination produced by its corresponding event. Do not
use source spans, debug variable names, equal types or matching counts to
choose among owners.

For both aggregates, require exact nominal definition, empty generic arguments,
variant zero, no annotation/active union field, typed destination and complete
declaration-indexed operands. Each operand consumes a unique staging local
defined by a Move from the already authenticated source binding. Staging
evaluation order must match canonical source initializer order, and occur
after prior source events and before the aggregate assignment. Whole Inner
staging is one record move, not two fabricated leaf moves.

Every later movement must read the complete typed Place projected from the
authenticated binding, and define exactly one fresh typed destination. Retain
its source path, destination and full Location. Check all source events in
order. Account for every Box, Inner and Outer MIR local, including staging;
unexpected owning storage or an alternate Box instantiation rejects.

Derive terminal cleanup from remaining source leaves: reverse lexical binding
order, then recursive field declaration order within each surviving record.
Match every Drop's full Place and Location exactly once. Group the two
declaration-ordered Inner leaves only when both remain under the same intact
Inner source prefix. That group retains the actual record Drop and both
constructor identities; each logical leaf projects its own storage Place and
the shared cleanup location. All other groups contain one scalar Box leaf.
An intact Inner may remain under Outer or in a moved local. No custom Drop is
allowed, so record cleanup is the compiler's ordinary field-drop glue, not an
unknown user effect. A moved subtree must not also be dropped under its old
parent. Authenticate the scalar pointer/read
producer from the selected local, after source events and before every drop;
Return follows all normal cleanup. All assignments are accounted for. Only
the existing completely unread constant Boolean/unit compiler-temporary rule
may discharge residual assignments, with its whole-body use audit.

## Evidence and tests

Return a private query-only certificate containing canonical construction
inputs, parameter/binding correspondence, typed aggregate/staging/movement
events, constructor-rooted terminal leaves, source exit/scope, scalar read,
ordered cleanup and full Return. Consuming projections may expose checked
operation inputs; an operation input alone must not be accepted as this body
certificate. Do not expose mutable MIR, public evidence constructors or a
public arbitrary-MIR verifier.

Ordinary consumers must assert exact event projections without reaching into
relation internals. Positives cover both nested leaves, all leaf cleanup
positions, reversed initializers, multiple extractions, whole Inner transfers,
shadowing and tail/explicit return. Source negatives and invalid Rust must not
publish evidence. A private corruption oracle alters owner/phase, parameter
anchors, complete nominal/projection types, staging sources/order, aggregate
operands, moved subtrees, read producer, missing/double/stale drops, Return and
otherwise-unaccounted operations; each edit must be non-vacuous and reject.
Exact compile-negative tests protect certificate/event/path privacy, prevent
operation-only erasure and reject arbitrary-MIR injection. Full isolated
historical/native/lint gates and a fresh broad review precede closure.

No C allocation, C layout copying, Java heap output, unsupported source
approximation or claim about all valid Rust programs follows from this proof.
