# M31-04 — Port and prove has-flag

- Status: complete

## Completion evidence

- One checked model implements the explicit-`argv` API with general PolyIR
  operations and 25 evaluator/native vectors.
- The differential oracle passes 42,273 admitted comparisons against the exact
  retained implementation.
- The port-specific gate passes all 16 targets: provenance, model,
  determinism, differential, eight generated packages, Java/C/C++ native
  consumers, style checks, and C/C++ sanitizers.

## Goal

Model the explicit-argument decision function once and prove it against the
pinned JavaScript implementation in every supported output.

## Definition of done

- The model exposes `has_flag(flag: String, argv: List<String>) -> Bool` with no
  has-flag-specific intrinsic.
- Prefix selection uses UTF-16 code-unit length exactly as upstream does.
- Candidate and `--` terminator positions use first-index `Option<I64>` values;
  matching after the first terminator is rejected.
- All 11 official assertions and expanded boundary vectors pass.
- A deterministic differential corpus covers broad Unicode flags, optional
  prefixes, empty strings, equals signs, duplicates, and terminator placement.
- Rust, TypeScript, JavaScript, Python, Go, Java, C++, and C packages compile,
  lint, and pass native tests from clean generated output.
- Three complete generations are byte-identical.

## Tests

- `bazel test //examples/real-world/has-flag:all --nocache_test_results --test_output=errors`
