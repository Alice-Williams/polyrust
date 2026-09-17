# Checked binary64 arithmetic

- Status: accepted staged contract; source admission not implemented
- Plan: [02N](../../plan/tasks/M35-03A-02N-floating-arithmetic.md)

## Semantics and boundary

Support only built-in f64 binary Add, Subtract, Multiply and Divide. The original
compiler HIR and TypeckResults must agree on both operands, result and built-in
identity; overloaded traits, coercions, other widths, casts, compound assignments,
remainder and fused operations are not implicitly admitted.

Observe exact binary64 bits except NaNs, which are compared by category.
Each operator rounds separately to nearest, ties to even, with gradual underflow.
Preserve signed zero, infinities and nontrapping exceptional results. NaN
payload/sign/signalling details, errno, floating exception flags/traps and
foreign changes to rounding/flush modes remain outside the supported contract.
Do not reassociate, contract into FMA, introduce integer conversion or replace
division with reciprocal multiplication.

## Layers

A private ArithmeticInput retains the canonical source expression and ordered
children. Its closed operator enum drives an executable FloatingArithmetic
mapping registered through each typed capability builder. Revalidate the
original compiler context at mapping entry. Lower/materialize the left operand
before the right operand, once each; preserve nested expression boundaries.

[C](languages/c/rust-floating-arithmetic.md) and
[Java](languages/java/rust-floating-arithmetic.md) own their exact AST choices.
Shared compiler semantics must not move into renderers. Certification keeps
whole-package authority, numeric safety, resources and dependency closure.
Renderers print existing structural operator nodes only. No runtime files.

## Independent evidence

Expected finite results use integer significands and exact rational arithmetic,
then one explicit binary64 rounding step. Signed-zero and nonfinite cases use
a separate exhaustive category table. Python float or native arithmetic must
not generate expected bits. Cross-check the oracle itself with golden halfway,
normal/subnormal and overflow cases and pinned native Rust.

Native target proofs add independent call traces and fault controls for wrong
operators, grouping, lost zero signs, reassociation/contraction and changed
operand evaluation. Original Rust-source and typed target fixtures are separate
layers of evidence; one does not replace the other.

References: [Rust operators](https://doc.rust-lang.org/reference/expressions/operator-expr.html),
[Rust f64](https://doc.rust-lang.org/std/primitive.f64.html), and
[Java SE 21 floating evaluation](https://docs.oracle.com/javase/specs/jls/se21/html/jls-15.html#jls-15.4).
