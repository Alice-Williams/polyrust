# M35-03A-02N-02 — C binary64 arithmetic target foundation

- Status: planned
- Parent: [binary64 arithmetic](M35-03A-02N-floating-arithmetic.md)
- Depends on: 02N-01
- Specification: [C mapping](../../specification/typed-generation/languages/c/rust-floating-arithmetic.md)

## Contract

Admit only exact F64 Add/Subtract/Multiply/Divide nodes in the existing shared C
profile. Preserve integer safety, operand authority and recursive validation.
Keep structural rendering unchanged. Verify supported binary64 execution/profile
requirements before claiming native arithmetic equivalence.

## Definition of done and tests

- Actual original/importing certificates admit each operator and composition.
- Mixed types, remainder, casts, unguarded integer arithmetic, forged imports
  and malformed nodes remain rejected at the appropriate boundary.
- GCC14/Zig O0/O2 compile and execute against 02N-01 exact expectations;
  prove signed zeros, gradual underflow, overflow, nonfinite categories and
  separate-operation rounding. Wrong operator, grouping, operand order,
  dropped/duplicated calls and fused-result faults must be detected.
- Target numeric, call, stack, source-byte and dependency checks remain active.
  Pure operators introduce no math-library or runtime dependency.
- Old bundle bytes, full Linux release/lint gate and independent review pass.
  This does not admit Rust source arithmetic yet.
