# M34A-08T — Infer typed-program feature requirements

- Status: in-progress
- Depends on: M34A-08S
- Blocks: M34A-10T and every remaining typed-language migration

## Goal

Replace the closed `StaticV1` profile and numbered-arity API with a consuming
typed builder whose result records exactly the portable features used by its
declarations and expressions.

## Definition of done

- Production Rust code contains no `StaticV1`, `StaticFeatureProfile`, or
  `static_program` public API.
- `TypedProgram<R>` and `ProgramBuilder<R>` have private invariant-bearing
  fields and cannot be forged by safe callers.
- Sealed feature markers cover every constructor exposed by the initial typed
  builder.
- Expressions and declarations infer structural requirement trees.
- `SupportsAll<R>` recursively requires explicit `Supports<F>` evidence for
  every inferred feature and has no permissive fallback.
- One shared `Nil`/`Cons` representation provides arbitrary typed parameter,
  local, argument, field, and constructor lists.
- Function and record APIs have no numbered arity suffixes and no public
  untyped list escape hatch.
- The checked/CoreIR replay bridge is private and treats rejection as an
  internal invariant defect.

## Tests

- Compile-pass a three-parameter function, exact three-argument call,
  three-field record, exact three-value construction, and field projection.
- Compile-fail wrong operand, return, argument type, argument count,
  constructor type, constructor count, cross-record field, cross-body local,
  protected name, wrapper forgery, and missing `Supports<F>` proof.
- Exercise every feature constructor and replay through the checker and CoreIR
  verifier.
- Prove repeated construction is deterministic.
- Run Rustfmt, strict Clippy, Buildifier, documentation tests, full tracked
  Bazel tests, and the release gate in the Linux development container.

## Commit gate

Commit and push `M34A-08T: infer typed program capabilities` only after every
named shared proof passes. Hosted CI for the exact implementation checkpoint
must be green before M34A-10T is marked complete.

## Exit evidence

Pending implementation.
