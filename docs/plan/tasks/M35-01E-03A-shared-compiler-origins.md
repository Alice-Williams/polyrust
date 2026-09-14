# M35-01E-03A — Shared compiler origin reader

- Status: complete
- Parent: [M35-01E-03](M35-01E-03-java-hir-mappings.md)
- Depends on: completed M35-01E-02

## Implementation contract

- Move rustc identity, location, visibility, resolved docs, module ancestry and
  public export extraction from c_lower/origin into source_origin. Reuse the
  existing RustSourceOrigin vocabulary; do not introduce another AST or parser.
- The shared module depends on rustc and codegen metadata, never a concrete
  backend. Its cache is local to one compiler analysis/lowering invocation.
- Keep CDeclarationKey, CIdentifier and CGeneratedOrigin assembly in C. Java
  will consume the same shared reader through its own typed registrations.
- Preserve all existing extraction limits, cache sharing, resolved namespace
  and alias graph behavior, remapped locations and doc-input verification.
- Declare the new files in every affected Bazel adapter/probe source set.
  A separate compiler-backed probe must build without C or Java dependencies.

## Definition of done and tests

- The backend-free probe runs after successful rustc analysis and explicit input
  verification, inspects function/type/field/module docs and identity, and passes
  the shared checked-metadata constructor. Omitted included-doc inputs reject.
- Existing C provenance, aliases/cycles/privacy, doc sharing, budget boundaries,
  source-dependency and native equivalence tests remain enabled and pass.
- Generated C/H artifact hashes captured before extraction remain unchanged.
- Full Linux Bazel gate, Rust/Bazel lint and fresh broad independent review pass.

## Scope boundary

No Java HIR mapping or new source capability is claimed by this extraction.
The reader returns metadata, not a compiler-authority or rendering certificate.

## Completion evidence

- Full container gate `063926b4-5267-41ec-a912-e2675aacbd97` passed 344/344
  tests across 415 targets, including the new backend-independent compiler test,
  all C/Java regressions and Rust/Bazel lint.
- All 31 generated C/header artifacts compare byte-for-byte unchanged by SHA-256
  against the pre-extraction baseline; no generated source was committed.
- Fresh Sol Extra High review found no core defects in dependencies, source
  inclusion, metadata/cache/budget/privacy/export preservation or proof coverage.
- The new test-only fixture walker initially encountered rustc's implicit std
  module. It now skips foreign definitions; production export handling did not
  change. The repaired probe also rejects omitted include_str! doc inputs.
- Optional review suggestions (generalize the fixed-fixture walker with a seen
  set; add more extraction-level module-limit probes) do not block this mechanical
  extraction. Existing production cycle handling and limits are unchanged.
