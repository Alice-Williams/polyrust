# ADR-0006: Static portable programs valid by construction

- Status: accepted
- Milestone: M34A
- Date: 2026-09-03
- Amends: ADR-0004 and ADR-0005 frontend and proof-boundary decisions

## Context

ADR-0005 makes the target renderer accept only an opaque package certified by
a language verifier. That is necessary for dynamic input, but it does not by
itself make a program authored through the Rust frontend fail at Rust compile
time when expressions, calls, declarations, or target capabilities are
incompatible.

PolyRust's primary initial use case is static authoring: generator programs are
written in Rust, compiled, and then used to emit multiple target languages. In
that setting the generic portable AST must expose enough Rust types that an
invalid program cannot be constructed through its safe public API. Runtime
validation belongs only at a genuinely unknown input boundary.

## Decision

The primary frontend produces `StaticProgram<F>`, where `F` is a closed,
compile-time portable feature profile.

- Expressions carry their portable result type as `Expr<T>`.
- Parameters, locals, fields, functions, records, interfaces, implementations,
  and methods are referred to through typed symbol handles, never source-name
  strings.
- Call and constructor handles carry their parameter and result types.
- Operator constructors accept only compatible `Expr<T>` values and return the
  operator's statically known result type.
- Target-neutral names supplied literally use compile-time checked constructors
  or macros. Target spellings remain linker-owned and collision-free.
- AST fields and invariant-establishing constructors are private. The static
  representation is neither deserializable nor directly mutable.
- A target opts into a profile with an explicit `Supports<F>` implementation.
  Static generation is callable only when `L: Supports<F>`.
- For such a bound, generic-to-target lowering and target rendering are total
  for user-caused syntax, typing, name, and unsupported-feature errors.
- Defensive CoreIR and target verifiers MAY remain in the implementation and
  test oracle. Rejection of a `StaticProgram<F>` on a statically admitted path
  is an implementation defect, not a user validation path.

The static phase graph is:

```text
typed Rust constructors/macros
  -> StaticProgram<F>
  -> TargetProgram<L, F> where L: Supports<F>
  -> RenderReadyPackage<L>
  -> RenderedSource<L>
```

The dynamic phase graph remains:

```text
UnknownInput
  -> UncheckedDocument
  -> CheckedProgram
  -> CoreProgram
  -> checked target pipeline
```

Successful dynamic conversion may produce the same internal typed program, but
the conversion necessarily returns diagnostics because its values were not
known when the generator was compiled.

## Universal versus optional features

`StaticProgram<StaticV1>` denotes the first common portable profile promised by
every language which declares `Supports<StaticV1>`. A richer program carries
a different feature-set type. Adding a backend which cannot implement an
optional feature does not weaken existing backends; generation for that
backend simply lacks the required trait implementation and fails to compile.

A convenience `generate_all` API may require all selected targets to implement
`Supports<F>`, thereby proving at the Rust call site that the program can be
lowered to every selected language.

## Soundness boundary

Rust proves that callers cannot mix expression types, forge symbol handles,
skip required construction states, or invoke a target lacking `Supports<F>`.
It does not prove that PolyRust's private lowering or renderer implementation
contains no bug. Those implementations form a small trusted base and remain
covered by compile-fail tests, native compiler/parser oracles, mutation tests,
and deterministic replay.

## Initial profile

M34A-08S introduces a deliberately bounded `StaticV1` profile with:

- portable compile-time identifiers;
- `Bool`, `I32`, `I64`, `F64`, `String`, and typed record values;
- records with exact typed construction and field access;
- pure functions with zero, one, or two typed parameters;
- typed calls, Boolean operations, equality/comparison, checked and wrapping
  integer arithmetic, floating-point arithmetic, and string concatenation; and
- expression bodies with statically matching return types.

Profiles expand monotonically. Interfaces, implementations, block-local scope,
collections, options/results, matches, tests, and the remainder of current
CoreIR receive their own compile-time constructors before the static frontend
claims complete portable coverage.

Java is the first `StaticV1` target in M34A-10S. The older dynamic Java route
remains for existing fixtures until they are migrated; it is not evidence for
the static contract.

## Consequences

- The current verbose builder is retained as the dynamic/compatibility
  frontend, not treated as compile-time-valid merely because it uses some
  typed handles.
- The static frontend may use a thin private bridge into checked/CoreIR during
  migration. That bridge must be unreachable with arbitrary unchecked input,
  must fail only on an internal invariant violation, and must eventually be
  replaced by direct total typed CoreIR construction.
- `RenderReadyPackage<L>` remains a useful target proof, but static target
  lowering creates it by preserving invariants rather than principally by
  discovering user errors at the bottom of the pipeline.

## Enforcement

- Rust compile-fail tests cover wrong operands, returns, calls, constructors,
  fields, protected literal identifiers, and unsupported target profiles.
- Wrapper-construction, mutation, and deserialization attempts fail to compile.
- Every compile-pass static fixture generates through every declared target and
  is accepted by that target's pinned compiler/parser without repair.
- Source policy rejects untyped/raw escape paths from `StaticProgram<F>`.
- Runtime negative tests remain mandatory for the separate unknown-input path.
