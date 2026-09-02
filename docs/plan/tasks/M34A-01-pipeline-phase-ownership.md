# M34A-01 — Enforce pipeline phase ownership

- Status: complete
- Depends on: M34A-00

## Goal

Create the typed one-way compiler skeleton and make phase bypasses impossible
through safe public APIs.

## Definition of done

- Shared phase types exist for checked input, verified CoreIR, unresolved target
  package, resolved package, private render view, rendered package, and
  validated manifest.
- Only the owning checker/lowerer/verifier/resolver/renderer/assembler can
  construct the next phase.
- `LanguagePlugin` exposes typed associated dialect/lowerer/resolver/renderer
  components; the object-safe registry stores a generic compiler adapter.
- A plugin cannot accept unchecked IR, render unresolved AST, inspect a render
  view it did not construct, or directly create an `OutputManifest`.
- Stable diagnostics preserve source provenance across every boundary.
- A minimal test dialect proves success/failure ordering and atomicity without
  committing a production language design.
- Legacy adapters remain explicitly quarantined until their language task and
  cannot implement the new compliance marker.

## Tests

- `bazel test //crates/codegen:typed_pipeline_test --nocache_test_results --test_output=errors`
- `bazel test //crates/codegen:typed_pipeline_compile_fail_test --nocache_test_results --test_output=errors`
- Fault-injection tests prove every failed phase prevents later phase calls and
  produces no partial manifest.

## Commit gate

Commit and push `M34A-01: enforce typed pipeline phases` only when all focused
tests and existing `//crates/codegen:all` pass in the dev container.

## Evidence

- `crates/codegen/src/typed_pipeline.rs` defines the sealed object-safe
  compiler adapter, verified Core/unresolved/resolved/render-view phases,
  renderer output, typed registry, sorted stage diagnostics, and manifest-only
  final assembly.
- Rustdoc compile-fail cases reject unchecked lowering, forged unresolved
  packages, unresolved rendering, and direct typed-compiler implementations.
- Unit fault injection covers every phase and proves no later phase runs after
  failure; invalid options stop before Core lowering; unsafe output paths fail
  atomically at manifest assembly; repeated compilation is byte-identical.
- Linux-container Bazel invocation
  `f7cdc364-f55e-4b37-817b-88d3db59114d` passed
  `//crates/codegen:all`, Buildifier, Rustfmt, and Clippy.
