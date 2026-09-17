# Java21 binary64 remainder mapping

- Status: specified; not admitted in the source-bound target body profile
- Parent: [shared remainder](../../rust-floating-remainder.md)

## Typed mapping

Use JavaBinaryOperator::Remainder with primitive Double operands/result and
Multiplicative precedence. Materialize operands left-to-right exactly once.
The structural renderer prints %. No import, helper, wrapper or runtime class
is required. Math.IEEEremainder is not equivalent and must not be substituted.

## Certification and proof

Both public verification and private source-bound readers validate exact types,
precedence and recursive operands. Imported callable identity/signatures and
resource/byte bounds remain authenticated. Integer remainder is not admitted
by this floating-point rule. Strict Java21 producer/importer/client compilation
must match the independent oracle, including zeros, NaNs, infinities, large
quotients and subnormal results; compiling family/order/value faults must fail.

[Java SE21, 15.17.3](https://docs.oracle.com/javase/specs/jls/se21/html/jls-15.html#jls-15.17.3)
defines the truncating quotient convention and nontrapping floating result.
