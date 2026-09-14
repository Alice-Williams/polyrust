# M35-01D-03A — Typed generated-file imports

- Status: complete
- Depends on: [M35-01D-02](M35-01D-02-c-direct-calls.md)
- Parent: [M35-01D-03](M35-01D-03-c-public-packages.md)
- Contract: [public packages](../../specification/typed-generation/languages/c/rust-hir-public-packages.md)

## Goal

Carry generated-header dependency evidence through the shared linker and C
language layer before any renderer can emit an include directive.

## Definition of done

- Add a shared resolved-file-import witness with private construction, exact
  target file identity and dialect-owned kind, derived from real file edges.
- Reconstruct imports at post-link verification; no caller-supplied lists or
  symbol aliasing introduced by a file import.
- Extend structural import formatting for the resolved file-import list.
- Add C standard/generated import variants, validated header paths and typed
  deterministic guards using existing CFileRef/CIdentifier ownership.
- Preserve default no-file-directive behavior and single-unit C behavior.
  Do not yet advertise complete public-package certification.

Header-owned file grammar/wrappers and resource accounting of include/guard
bytes are M35-01D-03B obligations. This step creates the typed dependency
vocabulary but keeps all header or multi-file packages rejected by the safe
projection/certification path. It cannot be used to render an unbudgeted header.

## Tests and proof

- Shared linker tests for one/many references, deduplicated destination files,
  missing/extra/retargeted/kind-mutated imports and forbidden visibility/cycles.
- Compile-fail tests prevent manufacturing a resolved import witness.
- C path tests reject directive injection, unsupported roles and invalid names;
  guard tests distinguish case/punctuation variants and retain exact identity.
- Existing Java and C unit/compile-fail gates plus Clippy, rustfmt, buildifier
  and documentation gates pass in Linux/Bazel.
- Read-only independent review; record accepted findings and gate IDs.

## Evidence and review decisions

- Shared linker owns `ResolvedFileImport`; canonical unresolved roots and helper
  expansion reconstruct exact reference provenance/multiplicity, file edges
  and imports. C kinds retain authenticated header identity, checked sibling
  paths and byte-encoded guards. No generated-symbol aliasing is introduced.
- Accepted review finding: deleting a linked reference, edge and import together
  could previously hide a dependency when item resolution consumed no spelling.
  The new regression reproduced false acceptance in invocation
  `fa58e07e-47d4-4dd2-9865-d5dc8baa7009` (82 passing, one failing test).
  `linking/reference_inventory.rs` now reconstructs the original inventory;
  coordinated deletion, changed provenance, multiplicity and helper-reference
  mutations are covered.
- Accepted review finding: new modules were omitted from focused policy inputs.
  Both Bazel filegroups and shell verifier argument/discovery paths were fixed.
  Failure injection proves partial/empty shared-module discovery does not run
  the verifier; positive checks prove the new paths actually reach it.
- The initial concern about missing header wrappers/resource accounting is not
  a D03A defect: those are explicit D03B obligations, and the safe projection
  still rejects header-only and multi-file packages. A dedicated regression
  proves this boundary. The task now states the split explicitly.
- Linux development-container gate `87836669-8e03-4f7f-b20a-34affe66e611`:
  15/15 targets passed: shared/C/Java unit and compile-fail tests, Clippy,
  rustfmt, buildifier, docs, source policy/failure injection, typed-generation
  policy, renderer policy and source-discovery proof. Cached results remained
  enabled; no test was disabled.
- First Sol Extra High read-only review rechecked both accepted repairs and
  reported no remaining core defect. Direct native generated-header rendering
  is D03B evidence, not claimed by this dependency-vocabulary step.
- A fresh independent Sol Extra High review found no core correctness or proof
  gaps. Optional additional ordering/deduplication cases were not treated as
  defects; the required exact-destination and mutation coverage is already green.
- Completion remains local; pushes are held for the complete C/Java migration.
