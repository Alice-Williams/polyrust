# M19 — Port slash with behavioral equivalence

- Status: complete
- Phase: 6
- Depends on: M17, M18

## Outcome

Reimplement the complete MIT-licensed `sindresorhus/slash` 5.1.0 package at
revision `98b618f5a3bfcb5dd374b204868818845b87bb2f` as one checked
PolyRust function and generate equivalent Rust, TypeScript, Python, and Go.

## Implementation checklist

- Pin the immutable MIT license, source, declaration, and official tests.
- Express the extended-length path guard and replacement with existing portable
  prefix, conditional, and literal-replacement semantics.
- Carry all official vectors plus prefix near-misses, UNC, Unicode, repeated
  separators, empty, and long inputs into permanent evidence.
- Differentially compare generated TypeScript with the pinned upstream.
- Run clean native format, lint/static check, compile, and test gates for all
  four generated packages.
- Prove deterministic regeneration and document the full result.

## Required exit evidence

- The full `slash(path: string): string` input domain is represented.
- Official and boundary vectors pass evaluator and all four targets.
- Exhaustive short-string differential inputs cover backslash, slash, question
  mark, drive punctuation, ordinary text, and Unicode.
- Extended-length paths are byte-identical while every other backslash becomes
  slash, including a large linear input.
- Three generations are identical and the full release gate remains green.

## Scope boundary

PolyRust generates a named function in each target rather than JavaScript's
default-export syntax. This is an API spelling difference, not a functional
behavior omission.

## Completion evidence

- The pinned MIT license, runtime source, declaration, and official test source
  are retained under `third_party/slash`.
- All 15 portable vectors pass the evaluator and generated Rust, TypeScript,
  Python, and Go native suites.
- The differential oracle passes 55,994 unique paths, including exhaustive
  short strings and two 90,000-backslash ordinary/extended inputs.
- Fresh packages pass Rustfmt/Clippy/Rust tests, Prettier/TypeScript/Node tests,
  Ruff/Mypy/Pytest, and Gofmt/Go vet/Go tests.
- Three independent generations are byte-identical.
- `bazelisk test //:release_gate --test_output=errors` passes all 29 targets,
  including all three real-world ports and repository Rust/Bazel linters.
- The implementation and proof are documented in
  [the compatibility-port report](../../ports/slash.md).
