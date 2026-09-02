# Backend author guide

> M34A migration notice: this guide describes the implemented M30 fragment API.
> New backends MUST follow the
> [typed-generation specification](specification/typed-generation/README.md);
> this guide will be rewritten as the new plugin API lands.

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
4. implements `LanguagePlugin` over `CheckedProgram` with a structured import
   key, dependency-bearing fragments, a runtime-helper graph, stable file group,
   and closed `LanguageSourceFile`;
5. implements a `LanguageRenderer` that sees only `ImportSet` and is the sole
   producer of dependency-directive syntax; and
6. registers through `BackendRegistry`, explicitly preflights, generates, and
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

The normative rules are the
[compositional target-language IR contract](language-ir-architecture.md); the
[compliance ledger](language-ir-compliance.md) records migration evidence for
the built-in targets. A backend is not compliant merely because its output
compiles.

New source backends should implement `LanguagePlugin` and make their public
`Backend::generate` method call `generate_with_plugin`. Translation owns symbol
allocation and flat mappings from portable types, declarations, expressions,
intrinsics, and capabilities to target constructs. It returns a
`LanguagePackage<Import>` with:

- stable file groups such as metadata, runtime, source, tests, conformance, and
  negative tests;
- `SourceFileRole` for every generated source file and `TextFileRole` only for
  metadata, documentation, and text assets;
- closed `LanguageUnit` values composed from dependency-complete target
  fragments for preamble, body, and epilogue syntax;
- target import and helper requirements attached to the exact fragment that
  introduces dependent syntax, then merged automatically by composition; and
- declared dependencies and injected helpers.

The associated `LanguageRenderer` receives only the sorted, deduplicated
`ImportSet`. It merges requirements and writes target syntax. It cannot inspect
`CheckedProgram`, choose semantic helpers, or add catch-all imports in
anticipation of possible output. A file with no imports must render without an
import section.

Checked-in runtime templates MUST be split into stable helper nodes. Each node
owns its body fragment, structured imports, and helper dependencies; translated
program/test fragments select roots and a deterministic closure emits only the
required nodes. Runtime templates MUST NOT contain preassembled import blocks.
Focused backend tests compare minimal and one-feature-at-a-time programs and
prove exact import and helper presence and absence.

The `Document` inside a fragment is target syntax IR, not permission to make
semantic decisions during rendering. A mapping selects names, target types,
helpers, and imports while it creates the fragment. Naked documents and
file-sized dependency repair passes are forbidden.

The source-policy test reads all backend production sources. Adding a new
checked-in target template or handwritten native fixture requires adding it to
the backend's `language_ir_policy_sources` filegroup. A fixture containing
literal dependency directives additionally requires a path-exact, reviewed
entry in `tools/source-policy/source_policy.py`; production templates never do.
