# M35-03A-02W-02 — C Unicode scalar target foundation

- Status: planned
- Depends on: [oracle](M35-03A-02W-01-character-oracle.md)
- Specification: [C17](../../specification/typed-generation/languages/c/rust-character-values.md)

## Contract

Represent checked scalar values as typed U32 expressions, parameters/results
and ordinary declarations. Reuse the existing uint32_t catalogue and inferred
stdint dependency; no C character token, wchar_t, raw fragment or runtime.

## Definition of done and tests

Before compiler admission, certify actual target packages for exact literal/
identity transport, immutable locals, conditional selection and six U32
comparisons. Run the full scalar corpus and comparison pairs through GCC14/Zig
O0/O2 and GCC UBSan with standalone headers and strict warnings. Detect actual
byte/UTF-16 narrowing and wrong-order faults. Authenticate graph, numeric and
call-effect evidence; adding U32 transport must not admit missing/indirect/
foreign callees or pointer storage. Prove correct inferred dependencies, resource
bounds, syntax/compile-negative controls and unchanged old output. Full Linux
release/lint and fresh broad review must pass before separate commit/push.
