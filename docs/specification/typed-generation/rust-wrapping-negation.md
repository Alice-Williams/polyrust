# Checked Rust wrapping signed negation

- Status: target foundation implemented (M35-03A-02H-01); compiler integration planned (02H-02)
- Implementations: [C17](languages/c/rust-wrapping-negation.md),
  [Java21](languages/java/rust-wrapping-negation.md)
- Plan: [M35-03A-02H](../../plan/tasks/M35-03A-02H-wrapping-negation.md)

## Semantics and scope

For signed width W, negate modulo 2^W: MIN maps to MIN; every other input maps
to its mathematical negative. The initial widths are i32 and i64. This follows
the explicit [Rust wrapping_neg contract](https://doc.rust-lang.org/std/primitive.i32.html#method.wrapping_neg),
not debug/release behavior of ordinary unary minus.

Do not infer arithmetic support from a name or silently replace checked/panicking
negation with wrapping. Other wrapping operations, widths, casts and user-defined
methods need their own capabilities and proof.

## Compiler boundary

WrappingNegation owns private compiler-checked input, canonical HIR, authenticated
built-in inherent method identity, exact signed-width enum and source operand.
Validate checked type, safe nongeneric signature and supported receiver
adjustments. A textual method name alone is insufficient authority. Resolve the
same actual DefId for method and associated-function syntax; reject unproven
forms. The pinned rustc remains the type/ownership checker.

## Mapping and rendering

Each plugin registers an executable typed mapping in its consuming builder.
The mapping returns a typed target value plus scope-owned evaluation declarations.
Materialize the receiver once before inspecting/reusing it. Imported calls in the
receiver keep their original producer witnesses and normal source order.

Renderer behavior does not change: print the certified normal target AST.
No copied source fragment, wrapping runtime or helper package is introduced.
Existing scalar signature/schema/domain identities and public/private boundaries
stay unchanged. Target certificates still prove syntax, binding, resource and
numeric safety; native truth tests provide separate equivalence evidence.

## Proof

Target foundation precedes compiler admission. Require guarded C numeric proof,
Java exact-width admission, boundary/deterministic native truth, deliberate
fault controls, typed receiver/order/shape probes and strict registration/input
contracts. Source admission must remain fail-closed until those layers exist.
