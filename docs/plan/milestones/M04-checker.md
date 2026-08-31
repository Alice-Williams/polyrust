# M04 — Implement resolver, type checker, and capability analysis

- Status: planned
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

## Scope boundary

Target reserved words, target support decisions, optimization, and evaluation.
