# Rust wrapping addition in Java21

- Status: target foundation implemented; checked Rust-source mapping pending 02Q-04
- Contract: [shared](../../rust-wrapping-addition.md)

## Typed lowering

Materialize the original left operand, then right, into exact primitive Int or
Long values. Construct JavaBinaryOperator::Add with Additive precedence and the
same primitive type on both operands and result. Do not box, narrow, widen,
coerce to floating point or call Math.addExact/Runtime helpers.

Java integer addition retains the low-width result on overflow, which matches
the explicit Rust wrapping operation; see
[JLS21 15.18.2](https://docs.oracle.com/javase/specs/jls/se21/html/jls-15.html#jls-15.18.2).
This does not authorize mapping ordinary checked/panicking Rust + to Java +.

## Certification boundary

Extend the checked source/dependency expression traversal for this exact
Int/Long addition shape, including precedence and recursive child checks.
Keep Double addition's existing contract separate. Reject mixed widths, boxed
numbers, booleans, strings, wrong result types and unrelated integer operators.
Keep body-byte/resource accounting and original dependency authority intact.
The renderer prints the already-supported structural operator; no string-based
source construction or special generated runtime is added.

## Required evidence

Primitive target AST/arity/type/precedence tests, safe negative certification,
original-call/import checks and strict separate Java21 producer/client compilation.
Compare signed 32/64-bit results against independent modular truth and native
Rust, including overflow boundaries and deterministic full-width values.
Detect narrowing, saturating, wrong-operation and operand-dataflow faults.
The source checkpoint additionally proves actual left-to-right once-only calls
and its private compiler witness/typed registration contracts.

The target foundation is certified by the focused wrapping-addition tests,
the direct dependency-reader cross-product and a separately compiled Java21
producer/consumer oracle over 15,790 input pairs. Five compiled faults distinguish
saturation, carryless addition, subtraction, narrowing and duplicated dataflow.
This target admission is not evidence that arbitrary Rust integer + is supported.
