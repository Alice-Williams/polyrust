# M35-03A-02C — Exact i64 values and comparisons

- Status: complete
- Parent: [M35-03A-02](M35-03A-02-scalar-parity.md)
- Depends on: M35-03A-02B
- Specification: [i64 source mappings](../../specification/typed-generation/rust-i64-values.md)

## Contract

Extend the existing no-heap source grammar with signed 64-bit values, without
adding arithmetic, casts, constants or ownership forms. Rust i64 maps to typed
C int64_t and Java long in literals, immutable local/parameter/field places,
explicit shared dereferences, direct-call signatures and six comparisons.
Mixed-width signatures are supported; mixed-width comparison operands are not.
The legacy single-function fn(i32) -> i32 entry mode remains unchanged. Package
mode is the primary proof path for broader signatures.

## Implementation order

1. Move integer-literal interpretation into a shared compiler-checked input.
   Use a closed typed value enum, private construction and checked range
   conversion, including the special negative-minimum literal shape. Both
   backends map the resulting enum, rather than reimplementing its semantics.
2. Extend each target's representation, function/record registration, dependency
   signature/import/export manifests, closed body admission and resource sizing.
   C uses a typed standard-type reference and deduplicated stdint dependency;
   Java uses its primitive long, not boxed Long or custom support routines.
3. Extend typed structural rendering and relevant platform obligations. Audit
   signed minimum spelling, exact 64-bit width and two-slot Java parameters.
   Keep other integer widths, arbitrary conversions and mutable integer writes
   outside the source/dependency admission grammar.
4. Add actual multi-crate Rust/C/Java packages and separately compiled consumers;
   cover scalar-field records, shared loads, local calls, exported aliases and
   imported long signatures. Add invalid/unsupported source and target-negative
   tests, plus constructor-privacy and typed mapping contracts.
5. Update bounded inventory coverage only after native tests pass; obtain fresh
   review, run the isolated full Linux gate, then commit/push this step alone.

## Definition of done and tests

- Minimum, maximum, zero, plus/minus one, values around both i32 limits, values
  above the exact-integer range of f64 and adjacent 64-bit boundary values agree
  bit-for-bit or by exact integer output against Rust and an independent oracle.
- All six comparisons cover ordered/equal operand pairs and preserve source
  evaluation order when nested in calls, conditions and lazy Boolean operands.
  Native test-copy tracing must observe operand and enclosing-call order;
  value-preserving reordered-comparison mutations must fail the trace oracle.
- Unsuffixed and suffixed literals agree, including -9223372036854775808i64;
  invalid overflow literals reject through rustc, never by truncation or panic.
- Separate C GCC/Zig O0/O2 and Java 21 -Xlint:all -Werror compilation succeeds.
  Consumers actually cross crate/package boundaries with exact signed values.
- Wrong-width manifests/references and unsupported u64/i128/f64, casts, integer
  arithmetic and writes reject. Absent/existing output stays atomic on failure.
- Existing i32/bool and short-circuit proofs pass unchanged where their contract
  is unchanged. No legacy runtime, helper catalogue or third-party dependency is
  added or removed. This is partial scalar parity, not all integer operations.

## Implementation and proof evidence

- Shared private LiteralInput construction decodes checked bool/i32/i64 values;
  C and Java map the closed enum. C uses a resolved int64_t standard symbol and
  exact minimum literal spelling; Java uses primitive long and two-slot bounds.
- The two-crate corpus checks 21 x 21 x 2 x 20 = 17,640 exact results against
  native Rust and an independent integer oracle. Java 21 strict lint and
  separately compiled GCC/Zig O0/O2 libraries/consumers agree. Deliberately
  narrowed return values and wrong-width manifest mutations are detected.
- The imported leaf function has one genuine `[i64, i32, i64, bool]` signature;
  the root passes i32::MIN between wide values, and the leaf uses that parameter
  in its choice. Independent metadata expectations check each exact position.
  A separate C pair fixture exposes only i32 while its private body compares
  i64 values: both required i64 assertions must appear in the header, and removing
  either must reject. These close the fresh review's two bounded coverage gaps.
- Review found that pure return values alone could not establish operand order.
  The repair instruments only test copies: L/R operand and C enclosing-call
  traces cover all six comparisons, a condition and lazy imported arguments.
  All six reordered-comparison mutations preserve values but produce the exact
  wrong RLC trace; each must fail the independent trace oracle on every target.
- Review also found that direct C target ASTs could bypass the source-only
  equal-width rule. The C package profile now rejects mixed-width comparisons
  and conversions involving i64; mutations use legal C AST construction.
  The review's analogous Java public-escape diagnosis was disproved by actual
  tests: operator_signatures.rs already rejects mixed operands at general AST
  certification. We retain the explicit Java dependency check as defense in
  depth, with separate public rejection and direct private-body tests. We do not
  broaden the general AST just to make a negative fixture reach a later gate.
  The proposed restriction of general C scalar-call effect analysis was not
  adopted: that analysis explicitly proves storage effects, not numeric or
  render admission. The actual profile remains authoritative. Existing 32-bit
  C normalization cases are retained for the unchanged capacity proofs.
- Eleven invalid/unsupported forms reject for both targets with absent/existing
  outputs (44 atomic checks). Java 255 parameter slots compile; 256 reject.
  Literal privacy compile failures and target platform/dependency/resource tests
  pass without widening unrelated source admission or changing existing limits.
- Regression testing caught unconditional i64 platform checks consuming old
  capacity budgets; requirements now follow non-assertion typed dependencies
  across both C files. Updated negative fixtures genuinely reach their intended
  rejection layers. A foreign re-export limitation is separately tracked in
  M35-03A-03; local aliases/imported calls do not claim to implement it.
- Exact tree `622088fa8cad0b1ef12cf536bc205b1b27d90041` passed all 552 tests
  across 708 targets in 537.360 seconds, invocation
  `223fb73a-a117-4479-a070-05064ce68cbc`. The isolated archive matched 2,427 Git
  blobs/modes. The fresh Sol Extra High review concluded with no remaining core
  findings on this exact tree. Completion-document changes are gated before push.
- Tested production artifacts are available outside the container in ignored
  `generated/m35-i64-622/c` and `generated/m35-i64-622/java`. No generated
  artifacts, legacy runtime deletions or disabled tests are part of this step.
