# Rust wrapping multiplication in Java21

- Status: target foundation and checked compiler-source integration complete
- Contract: [shared](../../rust-wrapping-multiplication.md)

## Typed lowering

Materialize left completely before right as matching Int/Long primitives. Build
JavaBinaryOperator::Multiply with Multiplicative precedence and equal operand/
result types. Java's low-width primitive product implements the explicit wrapping
operation; see [JLS21 15.17.1](https://docs.oracle.com/javase/specs/jls/se21/html/jls-15.html#jls-15.17.1).
No widening, narrowing, boxing, Double coercion or Math.multiplyExact/runtime call.
Ordinary panicking Rust multiplication remains a separate unsupported operation.

## Certification and proof

Dependency-body traversal admits the exact primitive shape, recursively checking
both operands and original call/import authority. Reject mixed widths, boxed,
String/Boolean types, incorrect result/precedence and unrelated operators. Preserve
existing floating operations, depth, call-height, arity and source-byte accounting.

Strict separate Java21 producers/forwarders/clients agree with independent modular
truth. Compiling addition, saturation, narrowing and disconnected-operand faults
must disagree at both widths. Source evaluation order and witness registration
are proved later at the compiler checkpoint, not inferred from target admission.
