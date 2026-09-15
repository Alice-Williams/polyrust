# M35-03A-02E — Eager Boolean operators

- Status: complete
- Parent: [M35-03A-02](M35-03A-02-scalar-parity.md)
- Depends on: M35-03A-02D
- Specification: [eager Boolean mapping](../../specification/typed-generation/rust-eager-booleans.md)

## Contract

Add built-in bool `&`, `|` and `^` through a separate EagerBooleans capability,
private compiler-checked input and executable C/Java builder mappings. Evaluate
left then right exactly once, even when the left value determines the result.
Keep lazy operators separate; do not broaden integer widths, overloads or casts.

## Definition of done and tests

- Exhaustive three-Boolean inputs compare Rust, independent truth tables and
  actual C/Java libraries/consumers, including nested lazy/eager combinations,
  negation, records, shared reads and an imported function in a second crate.
- Native test-copy traces prove operand order/count. Lazy substitution,
  duplicate-call and reversed-order faults retain values but fail traces;
  wrong-operator faults fail truth. GCC/Zig O0/O2 and Java 21 strict lint pass.
- Actual mapper AST probes verify C Bool operands -> Int operator -> Bool
  normalization, and Java Boolean operands/result with exact bitwise precedence.
  Probe and production output are identical.
- Fourteen compile-negative cases cover missing/duplicate/wrong capability,
  context, output, input and private-input construction across both targets.
- Target admission rejects mixed types, Boolean complement and unrelated
  arithmetic/shifts. Source rejection remains atomic for absent/existing output.
- Full isolated Bazel/lint gate, evaluated independent review, exported ignored
  examples and a dedicated green commit/push. Legacy entry points remain.

## Progress

Specification written before implementation. Full Boolean/catalogue parity and
runtime retirement are not claimed by this bounded step.

The first isolated candidate exposed a negative fixture whose leading Rust
block parsed as a statement, not the intended binary operand. Parenthesizing
that block makes it valid unsupported source and tests the intended admission
boundary. Old missing-slot fixtures are also updated to fill the new slot in
every builder, preserving their independent omission checks.

The corrected full gate on tree `11ee861c0b0e5d2736e58723868374f47be16ff8`
passed 587/588 tests (762 targets), including all eager feature proofs.
It found two stale Boolean-negation-suite expectations for now-supported
value-operand And/Or. Those forms are now explicit positive controls, with
reference variants retaining atomic negative coverage. No production mapping
change or test disable was needed. The corrected full gate passed as recorded below.

## Gate evidence

Exact corrected tree `88b6363e1c3a7f82caaf7a1c8d2f0e95111c6b2a` passed all
588 tests across 762 targets in 35.909 seconds, invocation
`1d1b442a-7533-47c9-8623-6039ef5acf69`. The isolated Linux archive matched
2,469 Git blobs/modes. Rust and Bazel lint passed; unchanged test results remain
cached. All 104 exhaustive native truths/traces, four mutation families, 15
actual mapped AST nodes per target, fourteen compile-negative contracts and
forty atomic eager-source rejections pass. Existing C/Java admission matrices
retain Boolean-complement, mixed-type, arithmetic and shift rejection.

The first independent Sol Extra High review accepted the test corrections and
found no remaining production/core defect or required test gap. A fresh blind
Sol Extra High review of exact tree 88b6363 independently found no remaining
core findings. No unrelated feature request was treated as a required fix.
Completion-document changes receive a final isolated gate before commit/push.

Native-tested production examples are exported, ignored and uncommitted, in
`generated/m35-eager-257/c` and `generated/m35-eager-257/java`. Their production
implementation is identical to the corrected tree: intervening repairs affect
test fixtures and documentation only. Full catalogue parity and runtime
retirement remain incomplete.
