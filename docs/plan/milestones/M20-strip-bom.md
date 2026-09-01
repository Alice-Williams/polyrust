# M20 — Port strip-bom with behavioral equivalence

- Status: complete
- Phase: 6
- Depends on: M17, M18, M19

## Outcome

Reimplement the complete typed behavior of the MIT-licensed
`sindresorhus/strip-bom` 5.0.0 package at revision
`b80d7bc94e79b4744d92a2dc6328c91d9afe9775` as one checked PolyRust
function and generate equivalent Rust, TypeScript, Python, and Go.

## Implementation checklist

- Pin the immutable MIT license, source, declaration, and official tests.
- Specify and implement reusable exact one-prefix removal semantics in the IR,
  checker, evaluator, Rust backend, and three target runtimes.
- Carry both official fixtures plus BOM boundary, Unicode, empty, and long
  inputs into permanent evidence.
- Differentially compare generated TypeScript with the pinned upstream.
- Run clean native format, lint/static check, compile, and test gates for all
  four generated packages.
- Prove deterministic regeneration and document the full result.

## Required exit evidence

- The full `stripBom(string: string): string` domain is represented.
- Official and boundary vectors pass evaluator and all four targets.
- Exhaustive short-string differential inputs cover leading, repeated, middle,
  and absent U+FEFF with Unicode and control-character neighbors.
- A large repeated-BOM input removes exactly one leading scalar.
- Three generations are identical and the full release gate remains green.

## Scope boundary

The upstream runtime rejects non-string JavaScript values, while its public
TypeScript API accepts only `string`. PolyRust represents that complete typed domain;
generated target APIs enforce their native string type rather than reproducing
an out-of-domain JavaScript exception message.

## Completion evidence

- The pinned MIT license, runtime source, declaration, and official test source
  are retained under `third_party/strip-bom`.
- All 18 portable vectors pass the evaluator and generated Rust, TypeScript,
  Python, and Go native suites.
- The differential oracle passes 55,991 unique strings, including exhaustive
  short strings and two 90,000-scalar leading/middle BOM inputs.
- Fresh packages pass Rustfmt/Clippy/Rust tests, Prettier/TypeScript/Node tests,
  Ruff/Mypy/Pytest, and Gofmt/Go vet/Go tests.
- Three independent generations are byte-identical.
- The port added the 59th v0 intrinsic and permanent checker/evaluator/backend
  tests for exact prefix removal.
- The proof exposed and fixed Go backend handling for U+FEFF string literals
  and keyword/predeclared-identifier shadowing.
- `bazelisk test //:release_gate --test_output=errors` passes all 33
  targets, including every real-world port and the Rust/Bazel linters.
- The implementation and proof are documented in
  [the compatibility-port report](../../ports/strip-bom.md).
