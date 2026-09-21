# M35-03A-02P — Runtime-free negative-zero composition proof

- Status: complete
- Parent: [scalar parity](M35-03A-02-scalar-parity.md)
- Depends on: [checked remainder checkpoint](M35-03A-02O-04-compiler-remainder.md)
- Specification: [negative-zero composition](../../specification/typed-generation/rust-negative-zero-composition.md)

## Contract

Provide a runtime-free source replacement for the existing FloatIsNegativeZero
behavior using ordinary Rust composition: v == 0.0 && 1.0 / v < 0.0.
Reuse existing checked comparison, short-circuit and binary64 arithmetic
mappings; do not recognize a magic function name or add a marker-only capability.
Retain the Apache-2.0 stdlib is-negative-zero 0.2.3 snapshot and its provenance.
The declared numeric API is in scope, not JavaScript non-number coercion.

This is a parity increment, not broad floating-family completion or permission
to delete the legacy implementation. Existing eight-language corpus gates stay
enabled; the new C/Java source proof is additive until corpus-wide cutover.

## Definition of done and tests

- Actual ordinary Rust input compiles natively and generates certified C/Java
  packages without runtime, wrapper, helper-package or math-library dependencies.
- Primitive f64 -> bool public API, crate identity, docs and private visibility
  match the source; exact artifact inventories reject extras.
- Native Rust, generated GCC14/Zig O0/O2, strict Java21, vendored upstream JS,
  and an independent raw-bit oracle agree over official vectors, both zeros,
  subnormals, every exponent, infinities/NaN categories and deterministic bits.
- Compiling zero-sign/guard/comparison faults are detected. A private
  source-owned reciprocal function makes the lazy branch observable; test-only
  instrumentation proves it executes only on zero, with value-preserving
  eager/duplicate-call faults detected.
- Existing operation-specific HIR/typed-AST/registration tests remain enabled.
  No new frontend form, borrowed overload, float constant family or bit-cast
  capability is silently admitted by the example.
- Export inspected unmodified packages, input Rust and baseline clients into
  the ignored local examples folder.
- Full isolated Bazel/lint gate, old-artifact/WIP preservation, fresh independent
  review, and a separate green commit/push precede completion.

## Implementation and initial evidence

The ordinary Rust source lives beside the existing example under
examples/real-world/stdlib-is-negative-zero/rust-source/lib.rs. Its private
reciprocal is an original source function, not an injected runtime helper.
No production compiler, target AST or renderer change is required.

The first native proof passed 86,017 unique binary64 inputs against the raw-bit
oracle, unchanged vendored upstream JavaScript, actual Rust and separate C/Java
consumers. GCC14/Zig run at O0/O2 and Java21 enables strict lint. Per-input trace
separators distinguish where reciprocal calls happen, not only total counts.
Three wrong-value and two value-preserving eager/duplicate-call faults are
detected. External Java access and C linking fail for the private reciprocal;
exposing it in disposable copies makes the same controls succeed.

The initial Bazel formatter failure was load ordering only and is corrected.
A dedicated source rustfmt target now accompanies Clippy. Exact API, imports,
documentation, owner identity, visibility and duplicate-declaration checks have
deliberate mutation controls. Subsequent gate and review evidence follows.

## Gate evidence

Initial complete source tree 6558c100df5392ef5a58a671293892bde92fcd42 passed
all 903 release/lint targets, 04fcbc1a-031c-4756-8969-78f3318a945d
(14 executed, 889 cached). The eager fault was then strengthened to hoist the
actual generated reciprocal call and reuse its result, giving exactly one
call on every input without changing any Boolean result.

Tree b8216ba92ad2d6d613bd9fddcbae69ab9e257915 passes all 903 targets,
2cd15d95-0b83-4b75-9ba4-d8e54bf12879 (4 executed, 899 cached).
All 348 previously generated files, including the completed remainder bundles,
remain byte-identical; all 38 preserved ownership/adjacent WIP hashes match.
Actual source, packages and baseline clients are exported and compared
byte-for-byte at generated/examples/negative-zero-b8216ba9/README.md.
Generated artifacts are ignored and not committed.

The independent Sol Extra High reviewer completed two audit passes on the
final source candidate with no actionable findings. The review covered the
entire bounded contract, actual generated artifacts, exact host export equality,
unchanged upstream provenance and all five compiling fault controls. No extra
feature was used as a completion condition. Final documentation closure and
the commit receipt identify the exact final tested tree.
