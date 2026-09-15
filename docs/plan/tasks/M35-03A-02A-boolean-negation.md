# M35-03A-02A — Boolean negation in Rust-source C and Java

- Status: complete
- Parent: [M35-03A-02](M35-03A-02-scalar-parity.md)
- Depends on: M35-03A-01
- Specification: [Boolean negation](../../specification/typed-generation/rust-boolean-negation.md)

## Contract

Add executable Supports<BooleanNegation> mappings for the built-in Rust bool
not operator to both new source backends. Keep the compiler-owned input contract
target-independent, and require the mapping in each consuming builder. Reuse
the typed target unary ASTs, certification and ordinary bundle publication.

## Definition of done and tests

- Admit bool operands only; reject integer bit-not, overloads, implicit reference
  adjustments and unsupported source syntax without publishing target output.
- Map literals, local/parameter/field places, explicit shared dereferences,
  nested negation, comparisons and supported direct calls.
- Preserve the operand's single evaluation and its existing source-order prelude.
- Test missing/duplicate/wrong capability or mapping signatures at compile time.
- Compile real Rust, C and Java libraries and consumers; compare truth tables,
  i32 boundary comparisons, fields, nested expressions and calls with an
  independent expected-result oracle. C GCC/Zig O0/O2 and Java lint must pass.
- Actual output inventories contain no custom runtime artifacts or dependencies.
- Full isolated Bazel/lint gate and independent review pass before commit/push.

This is the unary operation only, not full JavaBooleanLogic parity. Short-circuit
and/or need separate scope/prelude proofs and remain unsupported in this step.

## Completion evidence

- Both executable source mappings use checked bool-only input and the existing
  typed target ASTs. C profile, scalar-call evidence and structural rendering,
  and Java dependency/source-size admission all handle the new unary node.
- Fourteen negative registration/input contracts pass. Eight invalid/unsupported
  cases reject for both targets with absent and existing output (32 checks);
  valid controls publish. Integer bitwise-not remains outside this capability.
- Actual source-owned C/Java bundles and independently compiled consumers agree
  with Rust and an independent oracle on 8,204 inputs and 13 outputs: 106,652
  values per implementation, including GCC/Zig O0/O2 and strict Java 21 lint.
  Exact exported identities, file inventories and per-callee source counts pass.
- The first read-only review missed C's unhandled structural-renderer case;
  native/full tests exposed it. It was fixed together with stale rejection
  expectations and Bazel macro lint, and the repaired implementation was tested
  and reviewed again. No failing test was disabled.
- Fresh whole-scope Sol Extra High review found one remaining inventory/spec
  omission, which was accepted and repaired. Review closed with no remaining
  core findings on tree `aa1f3d2a01f422868e77bf4b8afc85d7231877ce`.
- The isolated Linux Bazel/native/lint gate on implementation tree
  `7bb02fd90d25ac761f9a640b077dccf7cfc289a7` passed all 529 tests across 671
  targets in 607.504 seconds. Invocation:
  `1d561d5d-4c60-4223-984f-1ca0c58f89b8`; archive validation checked 2,393 Git
  blobs and executable modes. The inventory and documentation-only closure is
  gated again before commit/push.
- Only partial BooleanLogic coverage is recorded. Legacy runtimes, callers and
  tests remain intact until the rest of their replacement coverage is complete.
