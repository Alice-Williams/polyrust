# M35-01D-03 — Checked public-header packages

- Status: complete
- Depends on: [M35-01D-02](M35-01D-02-c-direct-calls.md)
- Parent: [M35-01D](M35-01D-c-files-and-visibility.md)
- Contract: [public packages](../../specification/typed-generation/languages/c/rust-hir-public-packages.md)

## Goal

Use existing CFileRef/CSourceFile roles to produce a checked public header and
implementation for one crate without creating a second C AST.

## Ordered implementation slices

1. [M35-01D-03A — Typed generated-file imports](M35-01D-03A-typed-file-imports.md).
2. [M35-01D-03B — Whole-package projection/resources](M35-01D-03B-c-package-projection.md).
3. [M35-01D-03C — Rust public API package mapping](M35-01D-03C-rust-public-api-packages.md).
4. [M35-01D-03D — Integrated native/review proof](M35-01D-03D-public-package-proof.md).

Keep the first shared import work independently testable and all one-unit
regressions enabled. No child alone completes the public-package contract.

## Definition of done

- Extend the shared package projection and reconstruction to all registered
  files; authenticate the complete package once at each required phase.
- Register primary public declarations in GeneratedPublicHeader and definitions
  in their exact owning GeneratedSource. Private layouts stay in implementation.
- Derive file dependencies and standard includes from typed references. Add
  checked header guards and ordering in the language/link layer, not raw output.
- Produce a typed crate/file/API/linkage manifest and verify both directions
  against actual declarations and compiler export bindings.
- Public documentation follows primary header declarations; private item/field
  docs remain private. Module attachments retain exact file/crate ownership.
- Measure every file and aggregate package/call resources before certification.

## Tests and proof

- Compile implementation and an independent C consumer separately; the consumer
  uses only public headers, including each header twice in both include orders.
- Reject mutated crate owners, definitions in the wrong source, widened access,
  private layouts/dependencies in public headers and public test symbols.
- Assert exact includes, exports, alias bindings, documentation placement and
  three-render determinism; preserve original single-unit regression coverage.
- Strict GCC/Zig/sanitizer matrix, style, independent review and release gates.

This checkpoint does not prove dependency-crate generation or shared-library
dynamic export hiding; those require their explicit integration contracts.

## Completion

All four child slices are complete. Integrated gate
`a3f68a1f-6b51-4b51-9f05-cb5593a78996` passes all 308 tests, including the
24 native public-consumer configurations, historical language examples and C
capacity partitions. Fresh broad integration review found no core defects.
The ignored local package byte-matches the tested Bazel tree artifact. Continue
M35-01D-04; no push until the complete C/Java migration gate is green.
