# M11 — Implement required TypeScript backend

- Status: complete
- Phase: 3
- Depends on: M08, M09

## Outcome

Generate a strict, readable ESM TypeScript package for the full v0 IR, including
explicit 64-bit integer and Unicode semantics.

## Implementation checklist

- TypeScript name/keyword handling, escaping, imports, type mapping, and lowering.
- Constant lowering, interface/implementation generation, restricted contract
  dispatch, and native portable-test generation.
- Discriminated representations for enums, `Option`, and `Result` that distinguish
  absent values from null/undefined.
- `bigint`-based `i64` helpers and checked/wrapping `i32`/`i64` helpers.
- Unicode-scalar validation/iteration and immutable list helpers.
- Strict `tsconfig`, package metadata, generated test/runner support, and API
  design document.

## Required exit evidence

- `strict` type checking is enabled; generated public APIs do not use implicit
  `any`.
- `i64` never passes through JavaScript `number`.
- Tagged unions are exhaustively handled in generated functions.
- Explicit PolyRust contract implementations are type-checked even though
  TypeScript interfaces are structurally typed.
- Every portable test is emitted to the selected native test runner.
- Public list types are readonly and generated operations do not expose shared
  mutable identity.
- ESM/import output is deterministic and supported Node/TypeScript versions are
  pinned in CI.

### Verification

- Unit/golden tests for reserved names, escaping, astral/surrogate cases, bigint
  boundaries, enum tags, contracts/implementations, portable tests, nested
  generics, imports, and every lowering case.
- Type-level negative tests proving invalid variants/values are rejected.
- Native generated-package checks (exact package manager selected in M01):

```text
npx prettier --check .
npx tsc --noEmit
npm test
```

- Boundary execution tests for `i32`/`i64` overflow, negative zero/special floats,
  scalar string length, and list non-aliasing.

### Completion gate

All v0 fixtures are deterministic, native format/type/test checks pass, the
backend passes the contract suite, public API snapshots are reviewed, and at
least 20 evaluator vectors agree with generated TypeScript.

## Scope boundary

Plain JavaScript output, CommonJS, browser bundling, DOM APIs, async, and arbitrary
npm dependencies.

## Exit evidence

- `org.polyrust.typescript` consumes only `CheckedProgram`, declares every v0
  capability, and emits a deterministic dependency-free strict ESM package.
- Generated API declarations use readonly records/lists, explicit contract
  `implements` clauses, discriminated `Option`/value-`Result`, and `bigint` for
  every `i64`; the embedded checked document stringifies wide integer/float
  bits before JavaScript parsing.
- The generated native gate passed Prettier 3.9.6, strict TypeScript 7.0.2,
  TypeScript compilation, Node 24.20.0 tests, the invalid-tag negative fixture,
  one native test per portable test, and 20 semantic boundary vectors.
- `cargo fmt --all -- --check` and workspace Clippy with warnings denied passed.
  The authoritative `bazel test //...` gate passed all 21 tests across 47
  targets, including Rust/Bazel linters, dependency boundaries, all earlier
  gates, backend unit tests, and native generated-package execution.
