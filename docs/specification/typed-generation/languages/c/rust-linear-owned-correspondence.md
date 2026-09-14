# Closed linear owned-place correspondence

- Status: implemented compiler-only M35-02B-02 proof; no target heap admission
- Parent: [owned construction](rust-owned-construction.md)
- Plan: [linear owned places](../../../../plan/tasks/M35-02B-02-linear-owned-places.md)

## Initial admitted body

After successful rustc analysis, inspect a canonical ordinary safe, non-generic
Rust function with one immutable i32 parameter and i32 result. This is a narrow
experiment shape, not a new arity limit on general functions or backends.

Its unlabeled root block contains one initialized immutable Box binding, then
zero or more immutable whole-value move bindings. The first initializer is the
authenticated Box::new operation from B-01 and its argument is exactly the
resolved parameter binding. Each subsequent initializer resolves directly to
the preceding owned binding. The tail expression reads i32 by dereferencing
the last binding. No nested blocks, branches, calls beyond the constructor,
additional owners, mutable bindings, explicit returns or unimplemented
adjustments are admitted here. Limit owned bindings to 128 as a resource budget.

Binding HirIds, not names, identify these operations. Shadowing names is allowed.
Canonical function parameters are related to MIR argument locals by the
compiler-defined argument order and matching signature, not debug variables.

## Complete typed relation

Borrow the same owner's drop-elaborated MIR at the pinned PostCleanup phase.
The public admission entry reads that query itself; it must not accept an
arbitrary caller-supplied MIR body as checked input.

Require one finite normal-control-flow path from START_BLOCK to Return covering
every block. Only the authentic constructor call, the final owned drop,
unconditional edges and return are permitted terminators; abort-mode call/drop
unwind edges must be Unreachable. Extra calls, branches, cycles, unreachable
blocks or cleanup blocks diagnose.

The single constructor must match HIR by actual FnDef, GenericArgsRef and result
type. Trace its i32 argument through complete, uniquely defined scalar copy/move
assignments to the actual parameter local. A same-type constant or another
producer is not equivalent evidence. Its destination authenticates the first
owned binding's MIR place.

Follow the complete graph of typed whole-Box move assignments from that
destination. Each source binding's move must have exactly one corresponding
source/destination edge; every edge and every Box local must be consumed exactly
once by this relation. No extra, cyclic, duplicate or substituted owner edges
are admitted. This is checked producer/consumer correspondence for the closed
shape, not a reimplementation of Rust's borrow checker.

Prove the scalar return reads from the final owned place using the pinned MIR
read/projection/cast sequence and actual compiler types. Account for every
non-storage assignment; unrelated arithmetic, writes or unmatched temporaries
diagnose. The pinned internal Box pointer-projection sequence is observation
evidence, not a C layout specification or a license to emit raw pointer casts.

The pinned read is the final Box's field-zero Unique/field-zero NonNull value,
transmuted to `*const i32`, then dereferenced into the return place. Normalize
each actual field type and compare it with the recorded projection type. The
NonNull wrapper's field is the compiler's non-null pattern over that pointer
type (`TyKind::Pat` / `PatternKind::NotNull`), not an ordinary raw-pointer field.
The compiler's [type representation](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/type.TyKind.html)
documents the pattern variant; the pinned executable fixtures are the version
oracle. No generated code depends on the layout of these compiler internals.

StorageLive/StorageDead and Nop are ignored by this correspondence inventory.
Their validity comes from the successfully analyzed compiler body, not a new
storage verifier here. Private MIR mutation tests test the specified relation,
not arbitrary malformed-MIR safety. No safe consumer can supply such a body.

Finally require the one drop to target that last owned place, after the scalar
read and before the normal return. The resulting private, session-bound
evidence retains the canonical HIR root scope/bindings and typed MIR move/drop
locations. A count, matching spelling, debug annotation, source span or
traversal position alone cannot manufacture that evidence.

## Proof boundary and tests

The implementation must include positive zero/one/multiple-move and shadowing
fixtures, plus valid-Rust rejections for every excluded shape tested. Mutation
tests operate only through private test hooks: change the body owner, argument
producer, move sources/destinations, final read/drop place, or insert/delete
relevant operations. Each must fail the correspondence validator, not merely
become a Rust parser/type error. Exposed evidence fields/construction remain
private and compiler-negative tests enforce that boundary.

Retain the constructor's executable typed binding proof and all historical
C/Java/no-heap tests. Structured HIR remains the future C rendering input.
M35-02B-03 owns extensions to multiple owners, scopes, branches, fields and call
boundaries. M35-02C/D still own actual allocation, cleanup and native proof;
this relation alone cannot certify target code or enable heap translation.

## Executable proof targets

`//experiments/rustc-frontend:owned_linear_test` runs five admitted functions,
seven valid-Rust exclusions, 24 private correspondence mutations, four additional
valid-Rust source mutations, an empty-inventory control and compiler E0382/E0502
controls. The private mutation entry exists only under the proof build cfg.
`owned_linear_private_test` requires exactly one E0451 for fabricated evidence;
unrelated compiler errors cannot satisfy it. `owned_linear_format_test` covers
production modules, tests and fixture; the compiler-adapter build runs Clippy
with warnings denied. Existing constructor registration contracts remain enabled.
