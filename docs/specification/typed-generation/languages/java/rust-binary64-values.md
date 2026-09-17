# Rust binary64 values in Java 21

- Status: target foundation and checked Rust-source mappings implemented and verified
- Contract: [shared](../../rust-binary64-values.md)

## Typed representation and rendering

Map f64 to JavaPrimitive::Double and an exact TypePlan, never Float, boxed
Double, a long payload field, Runtime or a generated support class.
A finite literal variant owns FiniteBinary64. Print exact hexadecimal floating
syntax with the double suffix and parenthesized negative sign, including -0.0.
No decimal host formatting or target bit-conversion method is required.

Java double uses binary64 ([JLS 4.2.3](https://docs.oracle.com/javase/specs/jls/se21/html/jls-4.html#jls-4.2.3)).
Its lexical grammar supports exact hexadecimal floating literals
([JLS 3.10.2](https://docs.oracle.com/javase/specs/jls/se21/html/jls-3.html#jls-3.10.2)).

## Admission and proof

Admit exact primitive transport, immutable fields/locals, f64 signatures and
six ordinary comparisons. Count double parameters as two JVM slots. Retain
original-owner dependency proofs, call-height and source/classfile budgets.
Do not widen integer-only unary/bitwise/wrapping admission accidentally.

Strict Java 21 native consumers compare raw bits for finite values, signed
zero and subnormals; NaN input proof checks classification/comparison rather
than payload identity. Consumer use of Double.longBitsToDouble and
doubleToRawLongBits is test instrumentation, not generated runtime machinery.
Arithmetic, casts, nonfinite constant construction and float methods remain
separate capabilities with their own semantic proof.

## Checked Rust-source mapping

TypePlan::F64 retains exact source representation independently of the target
JavaType::Primitive(Double). ObjectTypes, ordinary FunctionSignatures and private
immutable record fields map only rustc FloatTy::F64. Existing shared-borrow plans
preserve their referent while erasing references into immutable Java values.
This does not broaden the public reference ABI or the entry-harness signature.

LiteralValues consumes the shared finite witness in its original checked
function context and constructs JavaLiteral::F64. ScalarComparisons requires
matching representations, materializes left before right and emits one of the
six primitive comparison operators. Only Bool ordering uses the existing
integer conversion; Double comparisons introduce no conversion or helper.

Owner manifests use version 6 when described function parameters/results or
record fields include f64. Older schemas and bytes remain unchanged otherwise.
Slot accounting remains two slots per Double, including imported signatures;
native proof compiles the 255-slot boundary and rejects 256 slots atomically.
