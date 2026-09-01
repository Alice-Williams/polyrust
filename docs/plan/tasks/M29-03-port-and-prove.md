# M29-03 — Port and prove normalize-newline

- Status: in-progress

## Goal

Model both version-5 overloads once and prove them against the pinned
JavaScript implementation in all supported targets.

## Definition of done

- The model exposes explicit string and bytes functions without a
  newline-specific intrinsic.
- All 13 official valid assertions and expanded binary/text vectors pass.
- A deterministic differential corpus covers broad strings and arbitrary byte
  sequences, including invalid UTF-8.
- Rust, TypeScript, JavaScript, Python, Go, Java, C++, and C packages compile,
  lint, and pass native tests from clean generated output.
- Three complete generations are byte-identical.

## Tests

- `bazel test //examples/real-world/normalize-newline:all`
