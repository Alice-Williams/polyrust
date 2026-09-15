# Standard scalar Box clone evidence

- Status: operation identity complete; whole-body correspondence remains planned
- Plan: [M35-02B-03K](../../../../plan/tasks/M35-02B-03K-owned-clone.md)
- Parent: [owned values](rust-owned-values.md)

## Semantic boundary

The selected operation is the standard Clone implementation for Box<i32> with
the standard Global allocator. Its shared receiver leaves the original owner
alive and its result owns a separately cloned payload. It is neither an owner
move nor reference cloning. The standard library documents the distinct boxed
result in [Box::clone](https://doc.rust-lang.org/std/boxed/struct.Box.html).
Exact compiler APIs and representations are asserted on our pinned Rust 1.98.0,
not inferred from the moving online documentation.

## Operation input

Use a closed enum for method-call/autoref and explicit UFCS/shared-borrow forms.
Retain the canonical call and receiver/borrow nodes, body owner, standard trait
method DefId, instantiated arguments/signature, resolved concrete implementation
and full source/result Box type. Recognize Box and Clone through compiler item
identity; authenticate the actual selected implementation. Display names are
diagnostic text only. A matching signature or a method called clone is not proof.

Successful rustc analysis is mandatory. A private input may be obtained only by
querying canonical source in that compiler session. Executable Supports bindings
consume that input with typed context/output; missing mappings fail statically.
The optional builder slot must not change existing constructor-only clients.

### Closed source forms

The receiver is a direct local path of full Box<i32, Global> type in the same
body. Method syntax has exactly one shared autoref adjustment. Explicit UFCS
syntax supplies `&local`, whose pinned adjustment is one built-in dereference
followed by one shared reborrow. Both adjustment targets are checked. The call
result and direct callee have no coercion. Type aliases resolving to that exact
Box are accepted. Field/temporary receivers, preborrowed variables, autoderef
from a reference, function-pointer calls and result unsizing are not admitted.

The resolved concrete item must belong to Box's standard-library crate, map
back to the authenticated Clone trait method and instantiate a Clone impl with
that exact full Box self type. Retain both the source trait-call identity and
resolved Instance; do not substitute one for the other in later MIR matching.

### Pinned observations

Seven selected fixture functions assert real Clone resolution and PostCleanup
phase. Method syntax borrows the original owned local directly; explicit UFCS
adds a shared-reference reborrow stage. Both leave two ordered Box drops in the
simple read-clone fixture. Reference cloning resolves to a different concrete
implementation and leaves only the original Box drop. These are typed probe
assertions, not a body certificate or an allocation-event guarantee.

The separate capability fixture checks all 17 free functions, including the
observation cases and rejected receiver/identity/signature variants. Matching
counts are only test inventory; admission checks canonical nodes, full types,
adjustments, item identities and the normalized concrete implementation.

## Correspondence and rendering

Operation evidence alone does not prove body ownership. The later closed body
certificate links the receiver to the exact live owned place through the actual
shared-borrow staging. Its fresh destination carries a second cleanup obligation.
Retain complete normal trace accounting, ordered drops and canonical source exits.
Render structured HIR operations through the existing typed C AST; do not render
borrow staging or MIR edges as source-level gotos.

## Exclusions and remaining proof

Initially exclude reference clones, clone_from, arbitrary payloads, user Clone
implementations, custom allocators/destructors, generic or dynamic receivers,
raw Box conversions, leaks and unwinding. Rejection is explicit, not an
approximation. Later body admission is limited separately from operation inputs.

C/Java heap output stays disabled during this evidence stage. Compiler facts do
not establish physical allocation counts under optimization or allocation-failure
behavior. M35-02C defines the selected abort/failure mapping; M35-02D independently
tests native results, allocation/drop events, failure injection and sanitizers.

## Operation checkpoint

K-01 passed all 481 tests across 613 isolated Linux/Bazel targets and a fresh
independent broad review after strengthening its observation assertions. The
linked task records the exact tree, review repair and gate evidence. No body
certificate or C/Java heap support is implied by this checkpoint.
