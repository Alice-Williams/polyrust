# Generated-code review guide

Review a generated package as a public API translation of the checked portable
module. Never patch the output: change host builder code, portable semantics, or
the backend and regenerate. In every target, verify stable names and docs,
immutable/value-oriented boundaries, explicit contract dispatch, checked or
wrapping integer behavior as declared, deterministic file ordering, and native
execution of every portable test.

## Rust

- Public records/enums/traits follow Rust ownership conventions without exposing
  backend runtime JSON details.
- `cargo fmt`, Clippy with warnings denied, debug tests, and release tests pass.
- The generated crate forbids unsafe code outside the single audited runtime
  boundary documented by the [Rust backend](rust-backend-v0.md).

## TypeScript

- Public shapes are readonly, unions are discriminated, and portable `i64` is
  represented by `bigint`, never `number`.
- Prettier, strict `tsc`, positive tests, and compile-fail type tests pass.
- Runtime code does not use reflection or unsafe escape hatches.

## Python

- Public records are frozen dataclasses, contracts are protocols, and collection
  annotations use immutable tuple/bytes forms.
- Ruff formatting/lint, strict mypy, compileall, pytest, and negative typing
  tests pass.
- No public signature leaks `Any` or mutable list types.

## Go

- Exported names and value records are idiomatic; slices crossing public
  boundaries are copied to preserve portable value semantics.
- `gofmt`, `go vet`, and `go test` pass with the pinned SDK.
- Generated code imports neither `unsafe` nor `reflect`.

## Java

- Start with the [curated v0 and interface packages](../examples/generated/java/README.md).
  They contain actual generated source; Bazel regenerates and compares every byte.
- Follow the [Java specification](specification/typed-generation/languages/java.md)
  and [module map](specification/typed-generation/languages/java/module-layout.md):
  inferred capabilities select executable mappings, which produce typed Java
  AST. Linking derives imports and helpers; certification precedes structural
  rendering. Runtime declarations follow the same path.
- Review interface signatures, multiple conformance, concrete/interface dispatch
  and explicit composition. Interfaces with no implementations remain valid;
  their private uninhabited permitted subtype exposes no constructible value.
- Require pinned Java 21 `-Xlint:all -Werror`, separately compiled consumers,
  negative compilation, native behavior, snapshots and deterministic output.
- Keep API typing and target capacity distinct. `JavaResourceError` reports
  documented finite class-file/output limits; syntax or mapping defects are
  not normal user validation errors. Native class-file evidence supplements,
  but does not replace, the typed verifier and mapping contracts.

The automated reference is the
[`models-and-validation` native gate](../examples/models-and-validation/native_test.sh).
