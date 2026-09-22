# Rust signed widening in Java21

- Status: target foundation complete; compiler-source admission remains separate
- Contract: [shared](../../rust-signed-widening.md)

## Typed lowering

Materialize the original Int operand once. Construct a JavaExprKind::Cast with
primitive Long target/result and Unary precedence. Java's int-to-long widening
preserves the exact signed integer value under
[JLS21 5.1.2](https://docs.oracle.com/javase/specs/jls/se21/html/jls-5.html#jls-5.1.2).
Use the ordinary structural renderer; no helper, boxing or runtime invocation.

## Certification and evidence

Dependency-body certification requires exactly primitive Int input, primitive
Long target/result, Unary precedence and recursively certified operand. Reject
boxed/String/Boolean/reference/floating types, narrowing or same-width casts,
target/result mismatch and other precedence. Preserve import authority, arity,
call-height, depth and source-byte accounting through the cast operand.

Strict separately compiled Java21 producers/forwarders/clients agree with
independent signed truth. Compiling zero-extension, premature-narrowing and
disconnected-result controls must disagree. Source evaluation traces and private
compiler witness registration are proved later, not inferred from this admission.

## Implementation receipt

[02T-03](../../../../plan/tasks/M35-03A-02T-03-java-widening.md) certifies the exact
cast with seven focused cases, 14,000 private-reader combinations and independent
public controls. Actual direct-call and materialized-local packages agree with
73,890 inputs under normal/interpreted Java21 and kill three compiling faults.
All 969 release/lint targets pass, including 412 Java unit cases; review is clean.
Prior output bytes match. The renderer and compiler-source admission are unchanged.
