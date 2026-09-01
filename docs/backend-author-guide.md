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
- target import requirements collected on the exact source file that uses them;
- target-owned preamble, body, and epilogue documents; and
- declared dependencies and injected helpers.

The associated `LanguageRenderer` receives only the sorted, deduplicated
`ImportSet`. It merges requirements and writes target syntax. It cannot inspect
`CheckedProgram`, choose semantic helpers, or add catch-all imports in
anticipation of possible output. A file with no imports must render without an
import section.

Checked-in runtime templates should contain bodies rather than preassembled
import blocks. Their language plugin declares the imports required by that body,
just as it does for translated program files. Focused backend tests must compare
a feature-bearing checked program with a minimal program and prove both import
presence and absence.
