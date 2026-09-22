# Rust wrapping subtraction in C17

- Status: target foundation complete; compiler-source admission unchanged
- Contract: [shared](../../rust-wrapping-subtraction.md)

## Typed lowering

Materialize original signed operands left-to-right. For exact S=I32/I64 and
matching U=U32/U64, compute and materialize u=U(left)-U(right) in unsigned width.
Construct the existing typed guarded signed reconstruction:

    u <= U(MAX) ? S(u) : -1 - S(~u)

Both unsigned-to-signed conversions are in range on their respective paths and
the signed subtraction is representable, including MIN. Normalize promoted C Int
back to exact I32. The unsigned difference wraps by defined arithmetic; never
substitute overflowing signed subtraction or an out-of-range cast. This is our
lowering design using [C N1570 6.2.5/6.3.1.3](https://www.open-std.org/jtc1/sc22/wg14/www/docs/n1570.pdf).

## Certification and proof

Extend the bounded profile only for same-width internal unsigned Subtract, walking
both children. Keep public signed signatures and existing numeric/ownership/resource
proofs. Modular underflow must not create nonwrapping allocation/extent evidence.
Do not add a trusted formula whitelist or bypass the path-sensitive cast checks.

Exact AST/type and malformed guard/conversion tests accompany separate GCC14/Zig
O0/O2 producer/client compilation, standalone headers and GCC UBSan. Compare every
corpus result with independent modular truth; detect reversed subtraction, addition,
narrowing and disconnected operands. Dependencies derive from typed symbols, without
runtime/math-library support. Source admission follows a separate gated checkpoint.

## Test structure

Addition and subtraction reuse operation-parameterized, test-only integer body
and package fixtures. Their semantic shape assertions and required mutation cases
remain operation-specific. Shared fixture code is not a production lowering or
trusted certification path. Native clients stay handwritten test code; actual
producer packages are rendered from certified typed ASTs.
