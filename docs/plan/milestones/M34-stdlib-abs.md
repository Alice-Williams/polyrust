# M34 — stdlib abs 0.2.3 equivalence port

- Status: in-progress
- Phase: 6
- Depends on: M33

## Outcome

Port the complete typed behavior of Apache-2.0
`@stdlib/math-base-special-abs` 0.2.3 into one checked PolyRust model and
generate equivalent Rust, TypeScript, JavaScript, Python, Go, Java, C++, and C
packages.

The portable API is `abs(value: F64) -> F64`. Its representation-level meaning
is to clear the IEEE-754 binary64 sign bit and preserve all exponent and
fraction bits. This maps negative zero to positive zero, negative infinity to
positive infinity, and negative NaNs to the corresponding positive NaN payload.

## Architecture contract

M34 adds one target-independent operation:

`FloatAbs(F64) -> F64` clears bit 63 and leaves bits 0 through 62 unchanged.
This exact definition avoids target-specific differences in signed-zero and
NaN handling.

Every backend must lower the operation through the M30 compositional
target-language IR contract. The mapping that emits target syntax owns its
validated imports/includes and helper roots. A backend must not introduce a
package-specific intrinsic, fixed dependency inventory, raw directive,
capability repair scan, or target-dependent approximation.

## Implementation checklist

- Retain the public JavaScript entry point and implementation, independent
  bit-level JavaScript implementation, TypeScript declaration and declaration
  test, runtime/bitwise/native tests, C implementation/header, package
  metadata, README, NOTICE, and Apache-2.0 license from lightweight tag
  `v0.2.3` at commit
  `fbdc5b76328d9f376ea1851c0e6c84bde50278bf`.
- Verify every retained byte against its exact upstream Git blob ID without
  network access at test time.
- Add `FloatAbs` to serialization, checking, evaluation, and all eight
  dependency-complete target mappings.
- Retain every official typed numeric assertion and add exact-bit signed zero,
  normal, subnormal, infinity, signaling-NaN, and quiet-NaN vectors.
- Differentially compare exact binary64 input and output patterns with the
  retained bit-level JavaScript implementation through lossless transport.
- Generate, format, lint, compile, and natively test all eight packages,
  including public Java/C/C++ consumers and C/C++ sanitizers.

## Required exit evidence

- Provenance tests pass for all retained upstream blobs.
- Checker and evaluator tests cover the exact signature, signed zeros, normal
  and subnormal values, infinities, NaNs with both signs, and invalid
  signatures.
- Every backend has focused positive and negative dependency/lowering
  coverage.
- Every official and expanded portable vector passes in the evaluator and all
  eight generated targets.
- The differential oracle reports every admitted exact-bit comparison passing.
- Three complete generations are byte-identical.
- Uncached `//...` and `//:release_gate` pass in the Linux development
  container, including Buildifier, Rustfmt, and Clippy.
- The completed milestone is committed, pushed, and green in hosted CI before
  another repository is selected.

## Scope boundary

The retained TypeScript declaration admits exactly one `number` and returns one
`number`. PolyRust `F64` represents the complete binary64 input domain,
including infinities, signed zeros, and every NaN bit pattern. Dynamic
non-number JavaScript coercions are rejected by the declaration and are not
part of the portable API.
