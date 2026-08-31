# M04 — Implement resolver, type checker, and capability analysis

- Status: complete
- Phase: 1
- Depends on: M02, M03

## Outcome

Convert unchecked PolyIR into immutable `CheckedProgram` values and reject every
structural, naming, typing, contract-conformance, portable-test, control-flow,
purity, and capability error required by Core.

## Implementation checklist

- Declaration/local scopes and stable symbol IDs.
- Type resolver with alias-cycle detection.
- Expression/statement type checker and return-path analysis.
- Exhaustiveness and unreachable-arm checking for enums, `Option`, and `Result`.
- Contract declaration/implementation checking and restricted contract-position
  enforcement.
- Portable invocation/argument/expected-value test checking.
- Purity/effect checks and recursion rejection for v0.
- Required-capability collection per node, declaration, and program.
- Crate-private checked-program constructors.

## Required exit evidence

- All references are resolved before backends see the program.
- Every expression exposes a checked type.
- Checking accumulates independent diagnostics where safe instead of stopping at
  the first error.
- The checker never consults a concrete target backend.
- Capability output is minimal, deterministic, and traceable to requiring nodes.

### Verification

- Positive fixture using every Core construct and at least ten portable tests.
- Negative fixtures for duplicates, unknown names/types, alias cycles, wrong
  arity, type mismatches, missing returns, non-exhaustive/duplicate patterns,
  missing/extra/wrong contract methods, illegal contract storage/return/equality,
  invalid test calls/expected values, recursion, impure nodes, and ambiguous
  integer operations.
- Diagnostic code/span snapshots for every rule.
- Property tests that checked programs contain no unresolved IDs and all
  expressions have types.
- Capability-set golden tests and insertion-order determinism.

```text
cargo test -p polyrust-check
```

### Completion gate

No safe public constructor can forge `CheckedProgram`; the full negative corpus
has stable diagnostics; all v0 features yield documented capabilities; and the
checker processes bounded hostile fixtures without panic or excessive recursion.

### Completion evidence

Completed in the pinned Linux development image on 2026-08-31:

- `cargo test -p polyrust-check` passed 19 unit, positive, negative, golden,
  determinism, property, and hostile-input tests plus the compile-fail doctest
  proving `CheckedProgram` cannot be constructed through safe public fields.
- `bazel test //...` passed all 13 repository tests across 33 analyzed targets,
  including the hermetic checker test, downstream checker consumer, Rustfmt,
  Clippy, Buildifier, dependency boundaries, and native generated Rust/Go tests.
- The positive fixtures exercise every type family, all declaration categories,
  every expression and constant constructor, every statement and pattern family,
  concrete and contract dispatch, all 55 intrinsics, and ten portable tests.
- Property-style inspection asserts every expression in the positive fixture has
  a checked type and every local reference has a nonzero stable `SymbolId`.
- The negative corpus covers invalid/duplicate structure and names, unresolved
  references, alias and constant cycles, wrong types and arities, missing return
  paths, unreachable code, ambiguous-width integer operations, non-exhaustive
  and duplicate patterns, missing/extra/wrong contract methods, illegal contract
  storage/return/equality, invalid portable calls/expectations, direct/indirect
  recursion, and the 64-level semantic complexity bound.
- Independent resolver/type failures accumulate and diagnostics are sorted by
  source then code. Stable code/source signatures are asserted for the
  multi-error resolution fixture.
- Capability sets are ordered, minimal, traceable per node/declaration, and
  unchanged when declaration insertion order is reversed. All v0 capabilities
  are listed in `docs/checker-v0.md`.
- `//smoke/checker:checker_consumer_test` obtains a checked program only through
  `check_program`; the constructor and all proof-carrying fields remain
  crate-private.
- Purity is structural in v0: no effectful or unbounded-loop expression variant
  exists. Bounded list iteration is capability-tracked, and function, method,
  dynamic-dispatch candidate, and constant dependency cycles are rejected.

## Scope boundary

Target reserved words, target support decisions, optimization, and evaluation.
