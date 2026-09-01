# Backend author guide

A backend is an extension over public, versioned crates: `polyrust-codegen`
provides `Backend`, registry, preflight, manifest, and contract-test APIs;
`polyrust-check` provides `v0::CheckedProgram`; and `polyrust-ir` provides the
versioned `v0` nodes and capabilities. Backends must not depend on emitter
internals or add a target-name branch to the core.

Use the separately rooted
[`examples/external-backend`](../examples/external-backend/Cargo.toml) package as
the template. Its [implementation](../examples/external-backend/src/lib.rs):

1. defines a namespaced `TargetId` and supported `IrVersionRange`;
2. reports support for every capability before generation;
3. declares its option schema;
4. consumes only `CheckedProgram` and returns an in-memory `OutputManifest`;
5. registers through `BackendRegistry`, explicitly preflights, generates, and
   calls `check_backend_contract` to prove deterministic behavior.

Run the template proof with:

```sh
bazelisk test //examples/external-backend:external_backend_test
```

Before proposing a real backend, add target-native compilation and tests for
every generated portable test, negative tests for unsupported capabilities and
invalid options, deterministic manifest tests, and the shared backend contract
test. Follow the existing [Rust](rust-backend-v0.md),
[TypeScript](typescript-backend-v0.md), [Python](python-backend-v0.md), and
[Go](go-backend-v0.md) backend documents for target-specific precedents.

## Language translation and rendering

New source backends should implement `LanguagePlugin` and make their public
`Backend::generate` method call `generate_with_plugin`. Translation owns symbol
allocation and flat mappings from portable types, declarations, expressions,
intrinsics, and capabilities to target constructs. It returns a
`LanguagePackage<Import>` with:

- stable file groups such as metadata, runtime, source, tests, conformance, and
  negative tests;
- a role for every file;
- dependency-bearing `LanguageUnit<Import>` values for target-owned preamble,
  body, and epilogue syntax;
- target import requirements attached to the exact translated unit that uses
  them, then merged automatically by its source file; and
- declared dependencies and injected helpers.

The associated `LanguageRenderer` receives only the sorted, deduplicated
`ImportSet`. It merges requirements and writes target syntax. It cannot inspect
`CheckedProgram`, choose semantic helpers, or add catch-all imports in
anticipation of possible output. A file with no imports must render without an
import section.

Checked-in runtime templates should contain bodies rather than preassembled
import blocks. Their language plugin wraps each body in a runtime unit and
declares the imports on that unit, just as it does for translated program files.
Do not attach imports to a file separately from syntax; `LanguageSourceFile`
deliberately exposes no such API. Focused backend tests must compare a
feature-bearing checked program with a minimal program and prove both import
presence and absence.

The `Document` inside a unit is target syntax IR, not permission to make
semantic decisions during rendering. A mapping should select names, target
types, helpers, and imports before it creates the unit. Over time a backend may
replace coarse file-sized documents with finer declaration/expression nodes;
the dependency ownership and package renderer contracts remain unchanged.
