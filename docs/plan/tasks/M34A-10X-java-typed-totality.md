# M34A-10X — Close Java typed-shape totality gaps

- Status: planned
- Depends on: M34A-08V and M34A-10U
- Blocks: completion of M34A-10W, M34A-10R, and M34A-11

## Goal

Make every currently admitted Java-supported typed shape lower successfully,
without user-triggered invariant panics or new generic frontend restrictions.
An interface with no implementations is explicitly valid.

## Implementation order

1. Add focused typed regressions for restricted Object names, record accessor
   versus interface method collisions, distinct interface signatures which
   erase identically, and interfaces with zero implementations.
2. Make target symbol allocation resolve these conflicts deterministically.
   Preserve readable names where safe; allocate disambiguated names by typed
   declaration identity and use those symbols for declarations, bindings,
   projections, and both dispatch forms. Never merge distinct methods merely
   because their source names or erased signatures coincide.
3. Implement the mapping-owned uninhabited Java interface representation in
   [the language specification](../../specification/typed-generation/languages/java/unimplemented-interfaces.md).
4. Replace preflight rejection tests for these representable shapes with
   successful generation/compiler tests. Retain rejection of forged target AST
   and genuine invalid dynamic inputs.
5. Review every remaining Java-specific rejection against typed constructors;
   list unreachable cases with evidence and resolve any admitted counterexample.

## Definition of done

- All counterexamples generate without panic and compile as Java 21.
- Naming transformations preserve distinct declaration and method identities
  through separately compiled public consumers and concrete/interface calls.
- Unimplemented interfaces do not expose foreign mutable implementations or
  introduce fake generic conformance evidence.
- No new generic reserved-name restriction or implementation-count requirement.
- AST, linker, and renderer remain typed; no raw text escape hatch.
- Changes and tests are split into focused modules, aiming below 500 lines;
  substantial test fixtures do not join production Bazel source sets.

## Tests

- Positive typed construction/generation regressions for each listed shape.
- Native Java consumer tests for renamed fields/functions, independently
  implemented colliding methods, and generic-erasure collisions with differing
  parameter/result types; verify distinct behavior, not only compilation.
- Positive zero-implementation interface fixtures plus foreign-implementation
  and synthetic-instantiation compile-negative tests.
- Mutation tests for permits, synthetic visibility, enum constants, and method
  identity/signature tampering.
- All Java native/conformance tests, Rustfmt, strict Clippy, Buildifier, full
  tracked Bazel graph, release gate, and deterministic all-target conformance.
- Fresh uncapped Sol Extra High review; evaluate every finding explicitly.

## Commit gate

Commit and push a separately identified checkpoint after local proof. Do not
mark Java complete before hosted CI and the review have passed.
