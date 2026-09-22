# Checked Rust wrapping signed subtraction

- Status: oracle and C target foundation complete; Java/source admission planned
- Plan: [02R](../../plan/tasks/M35-03A-02R-wrapping-subtraction.md)
- Targets: [C17](languages/c/rust-wrapping-subtraction.md), [Java21](languages/java/rust-wrapping-subtraction.md)

## Semantics and boundary

Support actual core i32::wrapping_sub and i64::wrapping_sub in method/direct
associated form. For width W return the signed representative of left-right
modulo 2^W, independently of Rust overflow-check configuration. See the official
[Rust contract](https://doc.rust-lang.org/std/primitive.i32.html#method.wrapping_sub).
This does not admit ordinary -, wrapping_mul, saturating/checked arithmetic,
other widths, unsigned/mixed operands, casts or borrowed/generic/trait methods.
Previously supported ordinary functions named wrapping_sub retain direct calls.

## Compiler and mapping contract

WrappingSubtraction owns a private witness containing canonical source HIR,
ordered original left/right children, actual core DefId and exact I32/I64 enum.
Authenticate original TypeckResults, inherent primitive owner, safe nongeneric
Rust ABI/signature and unadjusted matching operand/result types. Revalidate the
same context when lowering; spelling is only an exclusion filter.

Each consuming backend builder requires the executable mapping, not a support
marker. Wrong capability/input/context/output, missing or duplicate registration
must fail compilation. Materialize the whole left operand before the right once
each; preserve original dependency identities, source-owned declarations, crate/
module boundary, visibility and docs. Unsupported input rejects before publication.

## Required evidence

Independent unbounded modular truth and a boundary/borrow/full-width corpus must
agree with native Rust under checked and unchecked builds. Target foundations
are gated/reviewed before source admission. C additionally proves all conversions
and signed arithmetic safe; Java keeps exact primitive width without boxing.

Subtraction is not commutative: reversed values and reversed evaluation order
need distinct controls. Check measured native Rust/target traces and actual typed
dataflow, including a value-preserving call-order fault. Compile wrong-operation,
narrowing and disconnected-operand controls. Check exact APIs, docs, imports and
external privacy with positive controls; export actual packages. No runtime source,
mandatory support library, raw AST text or new third-party dependency is added.
