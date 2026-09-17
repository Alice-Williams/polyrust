# M35-03A-02J-01 — Primitive C/Java binary64 negation

- Status: complete
- Parent: [floating negation](M35-03A-02J-floating-negation.md)
- Depends on: M35-03A-02I

## Contract

Admit only F64-to-F64 CUnaryOperator::Negate and Double-to-Double
JavaUnaryOperator::Negate in the closed source-owner target profiles.
Retain integer bitwise/negation rules, dependency origin authority, numeric
safety, platform obligations and source/classfile budgets. C scalar-call shape
analysis must explicitly recognize this node without enabling other float
operators. No renderer helper or new import is needed.

## Definition of done and tests

- Typed constructors/certification preserve exact operand/result types.
  Bitwise/logical operators on double, mixed types, conversions and binary
  arithmetic retain explicit rejection coverage.
- Separate certified producer/consumer packages render plain unary minus and
  compile with GCC14/Zig C17 at O0/O2 and Java21 strict lint.
- Independent sign-bit oracle covers signed zero, normal/subnormal boundaries,
  deterministic finite samples, both infinities and quiet/signaling NaNs.
  Check exact non-NaN bits and NaN classification, not NaN payload identity.
- Direct, imported and nested negation preserve exactly-once calls.
  Native instrumentation detects value-preserving dropped/duplicated calls;
  no-negation and zero-minus mutants must fail exact-bit proof.
- Existing binary64/source tests remain passing: this child does not yet admit
  new Rust source forms. Full Linux release/Rust/Bazel lint and fresh independent
  Sol Extra High review pass before a scoped commit/push.

## Review decisions and focused evidence

- The first C fixture placed an imported call directly under unary negation.
  Existing full-expression sequencing correctly rejected it. The corrected
  fixture registers an F64 local, initializes it with the call, and negates its
  read. No sequencing rule or production call boundary was relaxed.
- Corrected implementation tree: 180ef48f80572f6c6c9e7b515c0a6a84f0e7e052.
  Focused Linux Bazel invocation 1b752c6c-6016-45a7-919f-4044383aad46
  passed two C tests and three Java tests (54.111 seconds).
- Native cases use 278 binary64 inputs and four direct/nested/imported outputs
  each: 1,112 exact non-NaN-bit/NaN-category checks per successful executable.
  GCC14 and Zig run O0/O2; Java21 compiles producer and consumer separately
  with strict lint. Four test-only faults cover no negation, zero-minus,
  dropped calls and duplicated calls. The last two retain numeric values while
  changing exact traces from ABAB to ABB or ABAAB per input.
- Independent Sol Extra High review binary64_targets_review found no required
  defect on that corrected tree. Its optional exhaustive binary-operator matrix
  is deferred: closed fallthrough plus existing family rejection tests,
  counterfeit Java unary tests and native zero-minus controls already cover
  this narrow unary admission. No additional arithmetic is admitted by default.
- Full Linux Bazel test //... //:release_gate passed on that implementation
  tree: 1,193 targets, 786/786 test targets (122 executed, 664 cached),
  702.759 seconds. Invocation: 9faa3e4c-0533-4815-9450-5faade5bcd99.
  Rust formatting/Clippy, Bazel lint, source policy and existing native gates
  remain enabled. The final documentation-only tree is gated again before push.
- No renderer, compiler source admission, metadata schema or custom runtime was
  changed. This completes target proof only; 02J-02 remains the compiler step.
