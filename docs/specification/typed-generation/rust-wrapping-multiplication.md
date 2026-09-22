# Checked Rust wrapping signed multiplication

- Status: independent oracle complete; target and source admission planned
- Plan: [02S](../../plan/tasks/M35-03A-02S-wrapping-multiplication.md)
- Targets: [C17](languages/c/rust-wrapping-multiplication.md), [Java21](languages/java/rust-wrapping-multiplication.md)

## Semantics and boundary

Support actual core i32/i64 wrapping_mul in method/direct associated form.
For width W return the signed representative of left*right modulo 2^W regardless
of Rust overflow-check configuration, following the official
[Rust contract](https://doc.rust-lang.org/std/primitive.i32.html#method.wrapping_mul).
This does not admit ordinary *, saturating/checked arithmetic, other widths,
unsigned/mixed operands, casts or borrowed/generic/trait methods. Ordinary
same-spelled functions retain their original direct-call identities.

## Compiler and mapping contract

WrappingMultiplication owns a private witness containing canonical source HIR,
ordered original children, actual core DefId and exact I32/I64 width enum.
Authenticate original TypeckResults, inherent primitive owner, safe nongeneric
Rust ABI/signature and matching unadjusted operands/result. Revalidate the same
context during lowering. Spelling is only an exclusion filter, not authority.

Each consuming builder requires an executable typed mapping. Missing, duplicate
and incorrectly typed registrations fail compilation. Materialize left completely
before right once each. Preserve original call/dependency identities, source-owned
declarations, crate/module boundaries, visibility and documentation. Unsupported
constructs reject before publication. No custom runtime or raw target AST text.

## Evidence

Independent unbounded products and multiplication-specific boundary/cross-product
cases agree with checked/unchecked native Rust. C proves conversions and signed
reconstruction safe, with modular unsigned loss kept separate from size evidence.
Java retains exact primitive widths. Target foundations are gated and reviewed
before source admission. Mutation controls distinguish wrong values from wrong
evaluation order; commutativity is not permission to reorder original calls.
Compile native producers/clients separately, check exact APIs/docs/imports and
external privacy, and export actual generated packages at source completion.
