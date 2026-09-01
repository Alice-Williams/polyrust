# M27-03 — Model and prove parse-ms in eight targets

- Status: complete

## Goal

Encode the complete representable parse-ms v3 behavior once and prove it against
the retained upstream implementation and every supported generated language.

## Definition of done

- One checked function maps `F64` to a seven-`F64` `TimeComponents` record.
- All official assertions and negative symmetry cases are retained.
- Thirty portable vectors pass in the evaluator and every generated target.
- A deterministic corpus of 10,105 inputs agrees field-for-field with upstream.
- Rust, TypeScript, JavaScript, Python, Go, Java, C++, and C compile and execute
  their generated tests; C and C++ also pass style and sanitizer gates.
- Three complete generations are byte-identical.

## Tests

- `bazelisk test //examples/real-world/parse-ms:all --test_output=errors`.
- The suite includes model, differential, determinism, native, conformance,
  formatter/linter, Java, C++, C, and provenance targets.
