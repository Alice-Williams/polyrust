# M28-03 — Port and prove is-fullwidth-code-point

- Status: complete

## Goal

Model the complete version-3 range classifier once and prove it against the
pinned JavaScript implementation in all supported targets.

## Definition of done

- The model contains the complete upstream range expression without a
  project-specific intrinsic.
- All official assertions and expanded boundary/IEEE vectors pass.
- A deterministic differential corpus covers every range boundary and broad
  finite/non-finite numeric classes.
- Rust, TypeScript, JavaScript, Python, Go, Java, C++, and C packages compile,
  lint, and pass native tests from clean generated output.
- Three complete generations are byte-identical.

## Tests

- `bazel test //examples/real-world/is-fullwidth-code-point:all`
