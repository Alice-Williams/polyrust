# M17 — Port escape-string-regexp with behavioral equivalence

- Status: complete
- Phase: 6
- Depends on: M08, M10, M11, M12, M13, M14, M16

## Outcome

Reimplement the complete typed public behavior of the MIT-licensed
`sindresorhus/escape-string-regexp` package at revision
`cbc42403142c96923b482604e1f3d627b1956aff` as one checked PolyRust program,
then generate and natively test equivalent Rust, TypeScript, Python, and Go
packages.

## Implementation checklist

- Record immutable upstream revision, license, typed API, source, and official
  test vectors.
- Specify and implement a target-independent, literal `StringReplaceAll`
  operation through IR, checker, evaluator, builder, and all four backends.
- Express the upstream algorithm with the verbose Rust builder, without
  target-specific source or a backend name switch.
- Carry every official upstream vector into portable tests and add boundary,
  repetition, ASCII, control-character, and Unicode vectors.
- Differentially compare the generated TypeScript public function to the pinned
  upstream JavaScript implementation over an independently generated corpus.
- Generate fresh packages for all four targets and run each target's formatter,
  static checks, compiler, and native test framework.
- Document the port, the semantic gap filled, provenance, limitations, and the
  command that reproduces all evidence.

## Required exit evidence

- The pinned upstream is unambiguously MIT licensed.
- The typed `string -> string` API has no valid input behavior omitted.
- The upstream official vectors pass in the evaluator and all four generated
  packages.
- The differential corpus includes every ASCII scalar, every pair and triple of
  escaped characters, repeated metacharacters, empty input, controls, combining
  text, and non-BMP Unicode; every result is byte-identical to upstream.
- Every generated package passes its native formatter, linter/static checker,
  compiler, and tests from a clean temporary directory.
- Repeated generation is byte-identical.
- The complete repository release gate remains green.

### Completion gate

M17 is complete only when the checked program, evaluator, generated Rust,
generated TypeScript, generated Python, and generated Go all agree with the
pinned upstream oracle, documentation and provenance checks pass, and no
backend or test can be skipped successfully.

## Scope boundary

JavaScript's dynamic non-string `TypeError` is represented by PolyRust's static
`String` parameter type: non-string calls are rejected before generation.
Regular-expression construction and matching are not part of the upstream
function's typed API and are used only as an additional oracle assertion.

## Completion evidence

- The pinned `LICENSE`, `index.js`, and `test.js` are retained under
  `third_party/escape-string-regexp` at revision
  `cbc42403142c96923b482604e1f3d627b1956aff`.
- The reference evaluator passes all 18 portable vectors.
- `differential_test` compares the generated TypeScript public API with the
  pinned upstream source for 3,750 unique inputs and checks every result as an
  anchored Unicode regular expression.
- `native_test` regenerates into a clean temporary directory and passes
  Rustfmt/Clippy/Rust tests, Prettier/TypeScript/Node tests,
  Ruff/Mypy/Pytest, and Gofmt/Go vet/Go tests.
- `determinism_test` proves three independent generations are byte-identical.
- The final `bazelisk test //:release_gate --test_output=errors` invocation
  passed all 21 release-gate targets, including both Rust linters and all four
  M17 evidence targets.
- The implementation and proof argument are documented in
  [the compatibility-port report](../../ports/escape-string-regexp.md).
