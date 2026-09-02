# M33 — stdlib is-negative-zero 0.2.3 equivalence port

- Status: complete
- Phase: 6
- Depends on: M32

## Outcome

Port the complete typed behavior of Apache-2.0
`@stdlib/math-base-assert-is-negative-zero` 0.2.3 into one checked PolyRust
model and generate equivalent Rust, TypeScript, JavaScript, Python, Go, Java,
C++, and C packages.

The portable API is `is_negative_zero(value: F64) -> Bool`. It returns true
only for the IEEE-754 binary64 value with raw bits `0x8000000000000000` and
false for positive zero, finite nonzero values, both infinities, all NaNs, and
all other bit patterns.

The retained TypeScript declaration accepts exactly `number`. Official
JavaScript tests also call the implementation with non-number values to prove
its untyped runtime permissiveness; those calls are retained as oracle evidence
but are outside the declared API and the PolyRust `F64` domain.

## Architecture contract

M33 adds one target-independent operation:

`FloatIsNegativeZero(F64) -> Bool` identifies negative zero by IEEE-754 value
representation. It is not sign-bit testing: negative finite values, negative
infinity, and negative NaNs return false. It is not ordinary equality:
`+0.0` and `-0.0` compare equal but this predicate distinguishes them.

Every backend must lower the operation through the M30 compositional
target-language IR contract. The mapping that emits target syntax owns its
validated imports/includes and helper roots. A backend must not introduce a
package-specific intrinsic, fixed dependency inventory, raw directive,
capability repair scan, or target-dependent approximation.

## Implementation checklist

- Retain the JavaScript entry point and implementation, TypeScript declaration
  and declaration test, runtime and native tests, C implementation and header,
  package metadata, README, NOTICE, and Apache-2.0 license from lightweight tag
  `v0.2.3` at commit
  `766200b9eeea46b7f827ac7d63effa6bea65d896`.
- Verify every retained byte against its exact upstream Git blob ID without
  network access at test time.
- Add `FloatIsNegativeZero` to serialization, checking, evaluation, and all
  eight dependency-complete target mappings.
- Retain every official typed numeric assertion and the invalid-type evidence;
  add signed zero, normal, subnormal, infinity, and positive/negative NaN
  vectors.
- Differentially compare exact binary64 bit patterns with the retained
  JavaScript implementation through a lossless transport.
- Generate, format, lint, compile, and natively test all eight packages,
  including public Java/C/C++ consumers and C/C++ sanitizers.

## Required exit evidence

- Provenance tests pass for all retained upstream blobs.
- Checker and evaluator tests cover the exact signature, signed zeros, normal
  and subnormal values, infinities, NaNs with both signs, and invalid
  signatures.
- Every backend has focused positive and negative dependency/lowering coverage.
- Every official and expanded portable vector passes in the evaluator and all
  eight generated targets.
- The differential oracle reports every admitted exact-bit comparison passing.
- Three complete generations are byte-identical.
- Uncached `//...` and `//:release_gate` pass in the Linux development
  container, including Buildifier, Rustfmt, and Clippy.
- The completed milestone is committed, pushed, and green in hosted CI before
  another repository is selected.

## Completion evidence

- All 12 retained upstream blobs pass immutable offline provenance checks.
- The evaluator and all eight generated packages pass the 22 exact-bit
  portable vectors; the retained JavaScript oracle agrees on all 86,018
  differential inputs.
- The M33 suite passes 17/17 targets. The complete uncached repository gate
  passes 250/250 tests and the complete uncached release gate passes 227/227
  tests in the Linux development container.
- Hosted workflow
  [33599714820](https://github.com/Alice-Williams/polyrust/actions/runs/33599714820)
  passes implementation/documentation commit
  `7820c55a69de282018923c55f617f41d5e12bf0d` across both determinism hosts,
  cross-host manifest comparison, pinned/stable Rust, fast checks, the
  Windows host/container contract, and cache-cold/cache-warm complete release
  gates.

## Scope boundary

The TypeScript declaration is the admission boundary: exactly one binary64
number is accepted. JavaScript's permissive calls with strings, objects,
arrays, functions, booleans, `null`, or `undefined` are invalid according to
the retained declaration and do not justify a dynamic `Any` type in PolyRust.
The public numeric domain is otherwise complete; no finite value, infinity,
NaN payload, or signed zero is excluded.
