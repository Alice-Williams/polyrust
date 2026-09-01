# M18 — Port trim-newlines with behavioral equivalence

- Status: complete
- Phase: 6
- Depends on: M17

## Outcome

Reimplement all three runtime functions of the MIT-licensed
`sindresorhus/trim-newlines` 5.0.0 package at revision
`6980540ee683a660fd82cb1bda37bf1ebd989179` as one checked PolyRust
program, then generate and natively test equivalent Rust, TypeScript, Python,
and Go packages.

## Implementation checklist

- Pin the upstream revision, MIT license, source, declarations, and tests.
- Specify target-independent trim-start and trim-end-by-scalar-set intrinsics.
- Implement their checker, evaluator, and all four backend lowerings.
- Express `trimNewlines`, `trimNewlinesStart`, and `trimNewlinesEnd`
  without target-specific code.
- Carry every official functional vector into portable tests.
- Differentially compare all three generated TypeScript functions with the
  pinned upstream over boundary-exhaustive and large linearity corpora.
- Compile, lint, format, and test all four fresh generated packages.
- Prove deterministic regeneration and document the result.

## Required exit evidence

- All valid runtime string inputs and all three functions are represented.
- Every official functional vector passes the evaluator and four targets.
- The differential corpus covers empty input, every short combination of CR,
  LF, ordinary text, whitespace, Unicode separators, and non-BMP scalars.
- At least one 90,000-newline input agrees with upstream, while implementations
  remain single-pass/linear at each trimmed boundary.
- Each generated package passes its native formatter, static checks, compiler,
  and tests.
- Three generations are byte-identical.
- The complete repository release gate remains green.

## Scope boundary

Upstream TypeScript refines string-literal return types with conditional types.
PolyRust preserves the complete runtime domain and behavior, which is the
project's functional-equivalence contract, but exposes the portable
`String -> String` signature in every target rather than a TypeScript-only
compile-time evaluator.

## Completion evidence

- The pinned MIT license, exact runtime and test sources, and public TypeScript
  declaration are retained under `third_party/trim-newlines`.
- All 31 portable vectors pass in the evaluator and generated Rust,
  TypeScript, Python, and Go native tests.
- The differential oracle passes 107,851 unique inputs and 323,553 comparisons
  across all three public functions, including three 90,000-newline cases.
- Fresh generated Rust passes Rustfmt, Clippy with warnings denied, and tests;
  TypeScript passes Prettier, strict `tsc`, and Node tests; Python passes
  compilation, Ruff, strict Mypy, and Pytest; Go passes Gofmt, Go vet, and Go
  tests.
- Three independent generations are byte-identical.
- `bazelisk test //:release_gate --test_output=errors` passes all 25 targets,
  including both real-world ports and the repository Rust/Bazel linters.
- The implementation and proof are documented in
  [the compatibility-port report](../../ports/trim-newlines.md).
