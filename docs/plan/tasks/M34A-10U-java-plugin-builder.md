# M34A-10U — Build Java from registered feature mappings

- Status: planned
- Depends on: M34A-08U and M34A-10T
- Blocks: renewed Java design review and M34A-11

## Goal

Make the Java plugin a consuming, typed collection of executable mappings and
route the complete typed intrinsic surface through those mappings into the
verified Java AST.

## Definition of done

- `JavaPluginBuilder` has one typed slot per independently supported portable
  feature and `.support::<F>(JavaFMapping)` is its only support-registration
  path.
- The built Java plugin, rather than `JavaDialect`, satisfies
  `SupportsAll<R>` for typed generation.
- Every `JavaFMapping` implements a typed lowering function whose result is a
  Java AST node or typed Java expression plan.
- The Java lowerer invokes the registered mapping; an unused evidence-only
  handler is a failure.
- Java's dynamic capability registry cannot advertise a feature absent from
  the built plugin.
- The old `java_supports!` macro and all empty/manual `Supports<F>`
  implementations are deleted.
- All existing and newly exposed intrinsic families generate through the same
  linker, post-link verifier, total renderer, and Java 21 compiler gate.

## Tests

- Compile-fail Java plugin builds for every registration failure category in
  M34A-08U.
- Remove one Java mapping in a mutation fixture and prove both typed admission
  and dynamic preflight reject its use.
- Instrumented unit tests prove each registered operation handler is invoked.
- Generate three byte-identical complete-intrinsic manifests.
- Hermetic Java 21 compiles with `-Xlint:all -Werror`; native and conformance
  consumers exercise normal and edge-case semantics for every newly exposed
  operation family.
- Run every Java verifier, compiler-oracle, mutation, interface, snapshot, and
  conformance target plus the full repository/release gates.

## Commit gate

Commit and push `M34A-08U/M34A-10U: register executable Java mappings` only
after all local evidence passes. Mark complete only after hosted CI is green,
then return Java to the user for design review before starting C17.

## Exit evidence

Pending implementation.

