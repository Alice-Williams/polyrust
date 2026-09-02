# M33-03 — Port and prove stdlib is-negative-zero

- Status: pending

## Goal

Model the complete declared numeric API once and prove it against the pinned
JavaScript implementation in every supported output.

## Definition of done

- The model exposes `is_negative_zero(value: F64) -> Bool` with no
  package-specific intrinsic.
- Every official typed numeric assertion and expanded exact-bit vector passes.
- A deterministic differential corpus spans sign, exponent, and mantissa
  boundaries and transports every binary64 value losslessly.
- Rust, TypeScript, JavaScript, Python, Go, Java, C++, and C packages compile,
  lint, and pass native tests from clean generated output.
- Public Java/C/C++ consumers, C/C++ sanitizers, and three-generation
  determinism pass.
- The invalid dynamic JavaScript calls remain documented oracle evidence and
  are not silently presented as admitted PolyRust values.

## Tests

- `bazel test //examples/real-world/stdlib-is-negative-zero:all --nocache_test_results --test_output=errors`
