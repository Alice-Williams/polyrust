# M35-01D — Preserve Rust crate and API boundaries in C

- Status: complete
- Depends on: [M35-01B](M35-01B-c-hir-typed-bridge.md) and [M35-01C](M35-01C-c-hir-documentation.md)
- Contract: [crate boundaries and visibility](../../specification/typed-generation/languages/c/rust-hir-files-and-visibility.md)

## Goal

Flatten same-crate source files without widening public/private access or
merging distinct crates.

## Ordered implementation checkpoints

1. [M35-01D-01 — Resolved crate API inventory](M35-01D-01-rust-api-inventory.md).
2. [M35-01D-02 — Direct calls and private same-crate helpers](M35-01D-02-c-direct-calls.md).
3. [M35-01D-03 — Checked public-header packages](M35-01D-03-c-public-packages.md).
4. [M35-01D-04 — Separate-crate native proof](M35-01D-04-c-crate-linking.md).

Each checkpoint preserves the existing target types and remains separately
reviewable. Metadata extraction alone is not proof of C privacy or an exported
API. The parent remains incomplete until every checkpoint and the full migration
gate passes. Keep source, fixture and test modules focused; split substantial
compiler probes and Bazel declarations at their dependency boundaries.

## Definition of done

- Retain authenticated crate, source-file, logical module and visibility metadata
  on declarations and fields, including effective public re-exports.
- Use one logical C package per admitted Rust crate, normally one implementation
  and public header; preserve source provenance rather than source file layout.
- Use internal linkage for non-public functions/objects where possible and keep
  private layouts/docs outside public headers.
- Derive a typed public/private/crate-dependency manifest from linked references
  and validate it against definitions.
- Extend the admitted no-heap subset with exact direct-call/module shapes needed
  for boundary fixtures. Do not claim the complete portable Functions capability.
- Support separate dependency-crate generation/API linking for the boundary
  fixture, or keep the task incomplete; same-crate tests alone do not prove it.
- Keep unsupported shared-library packaging disabled until a verified export
  policy exists. Private headers are not a security boundary.

## Tests and proof

- Same-crate multi-file Rust/C parity after flattening, including distinct items
  with the same spelling in different modules/files.
- Two admitted crates compile as separate C implementations and link through
  their public API; private dependency access fails Rust checking.
- All restricted visibility forms, private parent modules, public aliases,
  nested/inline/path-selected modules, public/private fields.
- External C consumer using only public headers; exact export inventories.
- Negative mutations for cross-crate merging, owner substitution, widened
  visibility, private-layout leaks and public test symbols.
- Determinism, declared source/doc dependencies and module-documentation order.
- Rustfmt, Clippy, Buildifier, docs and required release gates in Linux/Bazel.
- Independent review and permanent evidence for every admitted boundary shape.

## Commit gate

Keep milestone checkpoints separate. Hold migration pushes until the C and Java
gates both pass, following the latest user instruction.

## Completion evidence

All four ordered child checkpoints are complete. The final
[separate-crate proof](M35-01D-04D-separate-crate-proof.md) executes the same
four Rust libraries as separately compiled/linked C implementations under the
full native matrix, with exact public/private/import/doc assertions and fresh
review closure. Linux/Bazel gate 5630aeea-7116-4b63-bfcf-5a07969cb6a2
passed 343/343 tests. Existing same-crate, compiler privacy, source provenance,
metadata agreement, capacity, deterministic output and atomic publication
regressions remain enabled. Inspectable ignored C bundles are byte-verified.
Java retrofit now owns the next implementation checkpoint.
