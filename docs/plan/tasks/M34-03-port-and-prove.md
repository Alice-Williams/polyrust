# M34-03 — Port and prove stdlib abs

- Status: planned

## Goal

Model the complete declared numeric API once and prove it against the pinned
bit-level JavaScript implementation in every supported output.

## Definition of done

- The model exposes `abs(value: F64) -> F64` with no package-specific
  intrinsic.
- Every official typed numeric assertion and expanded exact-bit vector passes.
- A deterministic differential corpus spans sign, exponent, and mantissa
  boundaries and transports every binary64 input and output losslessly.
- Rust, TypeScript, JavaScript, Python, Go, Java, C++, and C packages compile,
  lint, and pass native tests from clean generated output.
- Public Java/C/C++ consumers, C/C++ sanitizers, and three-generation
  determinism pass.

## Tests

- `bazel test //examples/real-world/stdlib-abs:all --nocache_test_results --test_output=errors`
