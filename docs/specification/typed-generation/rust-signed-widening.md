# Checked Rust lossless signed widening

- Status: independent oracle and C foundation complete; Java/source admission pending
- Plan: [02T](../../plan/tasks/M35-03A-02T-signed-widening.md)
- Targets: [C17](languages/c/rust-signed-widening.md), [Java21](languages/java/rust-signed-widening.md)

## Semantics and boundary

An exact i32 operand cast with `as` to i64 retains its mathematical signed value.
Rust specifies sign extension for widening a signed integer in its
[numeric-cast contract](https://doc.rust-lang.org/reference/expressions/operator-expr.html#numeric-cast).
Admit only this direction and these resolved primitive widths. Narrowing,
same-width, unsigned, Boolean/char, floating, pointer/reference and arbitrary
From/Into/trait calls remain outside this increment. The native oracle may use
From independently; that does not admit it into translated source.

## Compiler and mapping contract

SignedWidening has a private canonical HIR Cast witness containing the original
operand and expression. Original TypeckResults must establish unadjusted i32
input and i64 output; source spelling alone is not evidence. Revalidate the same
context and identities at lowering. Resolved primitive type aliases follow the
existing type contract; this does not add exported alias declarations.

A consuming builder requires the executable capability mapping. Missing,
duplicate or incorrectly typed mappings fail compilation. Materialize the
original operand exactly once before conversion. Preserve original source-owned
declarations, dependency identities, documentation and crate visibility.
Unsupported syntax fails before publication. No raw target text or custom runtime.

## Proof obligations

Independent exact integer truth and native Rust agree at both optimization/check
settings. Target fixtures prove signed rather than zero extension and detect
premature narrowing/disconnected results. Source traces prove once-only effects;
typed dataflow and hostile witnesses prove original identity. Preserve resource
bounds, exact APIs/imports/docs, external privacy and atomic rejection. Target
foundations are independently gated/reviewed before source admission; actual
generated packages accompany final integration.
