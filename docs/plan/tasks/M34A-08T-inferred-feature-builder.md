# M34A-08T — Infer typed-program feature requirements

- Status: complete
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

The removal and Java's first consumer migration form one atomic buildable
checkpoint: `M34A-08T/M34A-10T: infer typed Java capabilities`. Push it only
after every named shared and Java proof passes. Hosted CI for that exact
implementation checkpoint must be green before either task is marked complete.

## Exit evidence

- The consuming `TypedProgram<R>` builder infers structural requirements from
  arbitrary `Nil`/`Cons` parameter, argument, and field lists. All proof state,
  raw nodes, checker replay, and CoreIR replay remain private.
- Eleven dedicated compile-fail cases cover wrong operands, returns, argument
  types/counts, constructor types/counts, cross-record fields, cross-body
  locals, protected names, wrapper forgery, and missing target support.
- One positive suite invokes every exposed constructor and replays the result
  through the checker and CoreIR verifier; separate tests prove three-element
  lists and deterministic name allocation.
- Local Linux-container gates passed: focused shared/Java/policy 6/6
  (`2dd1b94a-5a9f-4ada-b103-d3b88c6807ca`), Rustfmt/strict
  Clippy/Buildifier/policy 4/4
  (`f71a3d2c-07e3-4ad0-9c21-f73701b4ee67`), tracked repository 299/299
  (`6441764b-7f6d-4ca8-8a3e-8dd7fa2495d6`), and release gate 236/236
  (`bde6d995-3432-4860-9750-93fb9f3cc825`).
- `cargo test --workspace --all-features --locked` passed with all unit tests
  and doctests, including the typed builder's compile-fail suite.
- Atomic implementation checkpoint
  `f7c1efef772d2f8d37061bdc11d3e47c9888e1c9` was pushed and remotely
  verified. Hosted CI run `33819834141` passed all eight jobs for that exact
  SHA, including both determinism environments, both Rust toolchains,
  cross-host manifest comparison, and cache-cold plus cache-warm release gates.
