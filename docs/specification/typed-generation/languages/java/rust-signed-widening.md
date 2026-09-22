# Rust signed widening in Java21

- Status: target foundation and checked source integration complete
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
compiler witness registration are proved separately, not inferred from this admission.

The source mapper reauthenticates WideningInput, lowers/materializes the original
operand once, verifies TypePlan::I32, and constructs TypePlan::I64 with primitive
Long target/result and Unary precedence. The required SignedWidening builder
slot supplies this executable mapping. Cast traversal preserves the original
operand's local/imported callable identities. Test-only typed/dataflow probes
must leave every production package byte unchanged.

## Implementation receipt

[02T-03](../../../../plan/tasks/M35-03A-02T-03-java-widening.md) certifies the exact
cast with seven focused cases, 14,000 private-reader combinations and independent
public controls. Actual direct-call and materialized-local packages agree with
73,890 inputs under normal/interpreted Java21 and kill three compiling faults.
All 969 release/lint targets pass, including 412 Java unit cases; review is clean.
Prior output bytes match. The renderer and compiler-source admission are unchanged.

[02T-04](../../../../plan/tasks/M35-03A-02T-04-compiler-widening.md) subsequently
adds the checked source mapping. All 988 release/lint targets pass and review is
clean; strict Java21 normal/interpreted native proof, measured operand calls,
typed dataflow, original API/privacy and atomic checks pass. Actual packages
are exported, and previous output bytes remain unchanged.
