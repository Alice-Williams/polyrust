# Rust binary64 values in C17

- Status: specified; target implementation pending
- Contract: [shared](../../rust-binary64-values.md)

## Typed representation and rendering

Map f64 to CScalarType::F64 / primitive double, never long double or float.
The finite literal variant owns FiniteBinary64. Render exact hexadecimal
significand/exponent syntax, with a parenthesized unary sign when negative.
The printer receives no preformatted token or runtime bit-conversion call.
Headers are derived from typed dependencies, including platform constants.

The C standard permits hexadecimal floating constants and describes floating
environment characteristics ([N1570, 6.4.4.2 and 5.2.4.2.2](https://www.open-std.org/jtc1/sc22/wg14/www/docs/n1570.pdf)).
This backend's initial profile requires Linux LP64 binary64 double: radix 2,
53 significand bits, exponent limits -1021/1024, subnormals, eight-byte size and
the pinned ABI alignment. Add typed, closed platform assertions; size alone
does not establish binary64. Keep the existing exact integer assertions.

## Admission and proof

Admit primitive transport, immutable fields/locals, exact f64 calls/results
and six comparisons. No float-to-integer conversion, float bitwise operation,
arithmetic or method call is inferred from representation availability.
Retain numeric and original-call authority checks for integer expressions.

Native proof runs pinned GCC and Zig with no fast-math/contraction, default
nontrapping floating environment and no flush-to-zero. Assert these execution
assumptions and test subnormal transport/comparison explicitly. Bitwise
consumer inspection uses memcpy in handwritten tests, never pointer punning
or a generated helper. Preserve finite bits and signed zero; check nonfinite
input classification/comparison without claiming NaN payload identity.
