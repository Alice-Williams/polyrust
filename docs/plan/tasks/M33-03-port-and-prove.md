# M33-03 — Port and prove stdlib is-negative-zero

- Status: complete

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

## Completion evidence

- One checked `is_negative_zero(F64) -> Bool` function uses only the general
  `FloatIsNegativeZero` operation and generates all eight packages.
- All five distinct official numeric runtime assertions and all three
  declaration-test numeric calls are represented; duplicates are shared.
  Twenty-two portable vectors add signed normal/subnormal boundaries,
  infinities, and signaling/quiet NaNs with both signs.
- The oracle reconstructs values from 16-digit hexadecimal binary64 patterns.
  It exhausts both signs, every exponent, five mantissa boundaries, and 65,536
  deterministic full-width samples: 86,018 unique inputs agree with the exact
  retained `index.js`/`main.js` implementation.
- The 17-target uncached suite passes all generated/native packages, Java and
  C++ public generated consumers, an external C public consumer, C/C++ style,
  ASan/UBSan, three-generation determinism, and immutable provenance.
- The retained dynamic non-number calls remain evidence under `third_party/`
  and are explicitly outside the admitted numeric model.
