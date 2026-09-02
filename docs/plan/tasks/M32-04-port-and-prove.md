# M32-04 — Port and prove split-on-first

- Status: pending

## Goal

Model the complete string-separator API once and prove it against the pinned
JavaScript implementation in every supported output.

## Definition of done

- The model exposes
  `split_on_first(input: String, separator: String) -> List<String>` with no
  project-specific intrinsic.
- Empty input, empty separator, and absence return an empty list; a match
  returns exactly the prefix and suffix around the leftmost literal occurrence.
- All six valid official assertions and expanded boundary vectors pass.
- A deterministic differential corpus covers ASCII punctuation, multi-scalar
  separators, overlaps, NUL, CR/LF, combining marks, BMP, and astral values.
- Rust, TypeScript, JavaScript, Python, Go, Java, C++, and C packages compile,
  lint, and pass native tests from clean generated output.
- Public Java/C/C++ consumers, C/C++ sanitizers, and three-generation
  determinism pass.

## Tests

- `bazel test //examples/real-world/split-on-first:all --nocache_test_results --test_output=errors`
