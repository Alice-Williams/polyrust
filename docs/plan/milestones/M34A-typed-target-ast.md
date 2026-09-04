# M34A — Typed target-AST architecture migration

- Status: in-progress
- Phase: 8
- Depends on: M34-02
- Blocks: M34-03 and every new compatibility repository

## Outcome

Replace dependency-complete executable text fragments with the accepted
ADR-0004 pipeline as amended by ADR-0005 and ADR-0006. Static Rust-authored
programs use the valid-by-construction path:

`TypedProgram<R> -> TargetProgram<P, R> where P: SupportsAll<R>
-> RenderReadyPackage<D> -> RenderedPackage -> OutputManifest`.

Unknown/dynamic input retains the fallible refinement path:

`CheckedProgram -> CoreProgram -> UnresolvedPackage<D> -> VerifiedPackage<D>
-> LinkedPackage<D> -> RenderReadyPackage<D> -> RenderedPackage ->
OutputManifest`.

Rust, TypeScript, compiler-derived JavaScript, Python, Go, Java, C++20, and C17
each satisfy their complete language specification. Known target operations are
strictly typed in Rust; dependencies and helpers are derived from typed
references; runtimes are structural AST; and executable source is emitted only
by a total structural renderer from an opaque language-certified package.

Portable interfaces, explicit conformance, first-class polymorphic values, and
composition work in every target. Portable inheritance remains absent. Java
and C++ may prove only the typed one-edge external-framework adapter exception.

## Task sequence

Shared layers:

1. [M34A-00 — accept specifications and baseline audit](../tasks/M34A-00-specifications-and-audit.md)
2. [M34A-01 — pipeline phase ownership](../tasks/M34A-01-pipeline-phase-ownership.md)
3. [M34A-02 — canonical CoreIR](../tasks/M34A-02-core-ir.md)
4. [M34A-03 — exhaustive capabilities](../tasks/M34A-03-capabilities.md)
5. [M34A-04 — target-AST framework](../tasks/M34A-04-target-ast-framework.md)
6. [M34A-05 — symbols and linking](../tasks/M34A-05-symbols-and-linking.md)
7. [M34A-06 — interfaces and composition](../tasks/M34A-06-interfaces-and-composition.md)
8. [M34A-07 — structural runtime/files/packages](../tasks/M34A-07-runtime-files-packages.md)
9. [M34A-08 — strict Handlebars rendering](../tasks/M34A-08-handlebars-rendering.md)
10. [M34A-09 — manifest verification and evidence harness](../tasks/M34A-09-manifest-verification.md)
11. [M34A-08R — intrinsic validity certificates and total rendering](../tasks/M34A-08R-intrinsic-validity-rendering.md)
12. [M34A-08S — static valid-by-construction portable AST](../tasks/M34A-08S-static-portable-ast.md)
13. [M34A-08T — inferred feature builder](../tasks/M34A-08T-inferred-feature-builder.md)
14. [M34A-08U — executable feature registration and complete intrinsic surface](../tasks/M34A-08U-executable-feature-registration.md) — complete

Language migrations:

15. [M34A-10 — Java](../tasks/M34A-10-java.md) — in progress
    - [M34A-10R — blind-review remediation](../tasks/M34A-10R-java-review-remediation.md)
    - [M34A-10V — intrinsic Java syntax validity](../tasks/M34A-10V-java-intrinsic-validity.md)
    - [M34A-10S — static portable-program generation](../tasks/M34A-10S-java-static-program.md)
    - [M34A-10T — inferred capability generation](../tasks/M34A-10T-java-inferred-capabilities.md)
    - [M34A-10U — executable Java plugin builder](../tasks/M34A-10U-java-plugin-builder.md) — complete; ready for user review
16. [M34A-11 — C17](../tasks/M34A-11-c.md)
17. [M34A-12 — Rust](../tasks/M34A-12-rust.md)
18. [M34A-13 — TypeScript](../tasks/M34A-13-typescript.md)
19. [M34A-14 — compiler-derived JavaScript](../tasks/M34A-14-javascript.md)
20. [M34A-15 — Python](../tasks/M34A-15-python.md)
21. [M34A-16 — Go](../tasks/M34A-16-go.md)
22. [M34A-17 — C++20](../tasks/M34A-17-cpp.md)
23. [M34A-18 — legacy deletion, historical replay, and release](../tasks/M34A-18-replay-release.md)

Tasks are dependency ordered. A task is committed and pushed only after its
listed tests pass in the Linux development container. Language tasks update
the ADR-0004 compliance ledger in the same checkpoint.

## Required exit evidence

- Every shared layer specification has its named compile-fail, fault-injection,
  unit, property, and deterministic evidence.
- Every typed Rust-authored example is a `TypedProgram<R>` with inferred
  requirements; invalid operands,
  calls, returns, constructors, fields, identifiers, and unsupported target
  profiles fail during Rust compilation.
- Every support proof comes from a plugin-builder slot containing the exact
  executable typed mapping; empty/manual `Supports<F>` implementations are
  absent.
- Every renderer accepts only an opaque render-ready certificate, is total over
  its closed AST, and contains no executable template or source escape hatch.
- The shared and all eight language rows in the typed-generation compliance
  ledger are **Pass**.
- There is no production executable raw-source node, runtime source constant,
  paired JavaScript source path, manual import/include requirement call, or
  plugin manifest bypass.
- Every known target callable is selected with typed enum/catalogue APIs and
  every used dependency/helper is resolver-derived.
- Interface declarations, multiple conformance, static and dynamic dispatch,
  interface values in every admitted type position, explicit composition, and
  ownership/value semantics pass in all outputs.
- All historical ports M17-M33 regenerate and pass evaluator, native, retained
  upstream oracle, and determinism tests through the new pipeline.
- Buildifier, Rustfmt, Clippy, all language linters/static analyzers, native
  compilers/tests, C/C++ sanitizers, uncached `//...`, and
  `//:release_gate` pass in the dev container.
- The final checkpoint is pushed and hosted CI is green before M34-03 resumes.

## Scope boundary

M34A changes representation, adds the accepted first-class interface surface,
and now exposes the complete already-defined PolyIR v0 intrinsic catalogue
through M34A-08U. It does not add new portable semantics, source-language
parsers, arbitrary inheritance, user-editable templates, or a new
compatibility repo.
The already completed M34-02 `FloatAbs` semantics are retained; the partial
uncommitted M34-03 package is not carried into an architecture checkpoint.
