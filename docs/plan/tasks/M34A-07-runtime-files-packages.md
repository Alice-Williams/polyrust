# M34A-07 — Make runtimes, files, and packages structural

- Status: complete
- Depends on: M34A-05

## Goal

Represent helpers and artifact layout as typed dependency graphs rather than
runtime source blobs or file-body documents.

## Definition of done

- Helper IDs and helper declarations are dialect-owned closed enums plus typed
  AST expansion functions.
- The linker computes deterministic transitive helper closure, emits each
  helper once, and rejects missing nodes/cycles/illegal placement.
- Helper calls use typed symbol references; helper bodies use the same AST and
  verifier as generated declarations.
- Package/file/group/role models structurally represent source declarations,
  tests, metadata, documentation, and assets.
- Imports/includes, declarations, and helper placement cannot be smuggled
  through file metadata or grouping.
- Source paths are validated, relative, collision-free, role-consistent, and
  deterministically ordered.
- C/C++ declaration/definition/complete-type placement and Java one-public-type
  rules are expressible by the framework.
- JavaScript package source roles are reserved for TypeScript compiler outputs.

## Tests

- `bazel test //crates/codegen:runtime_helper_graph_v2_test --nocache_test_results --test_output=errors`
- `bazel test //crates/codegen:typed_package_test --nocache_test_results --test_output=errors`
- Minimality, closure, missing/cycle, placement, path traversal/collision,
  source-role bypass, and deterministic file-order fault-injection tests.

## Commit gate

Commit and push `M34A-07: make helpers and packages structural` only after
focused/shared package and security tests pass in the dev container.

## Exit evidence

- `RelativeOutputPath` is a validated opaque type. Source and non-source
  artifacts use disjoint Rust types; normalized case-insensitive collisions,
  traversal, absolute/drive/UNC-style paths, control bytes, ambiguous
  separators, and reserved output/device names are rejected.
- Source roles, file-group roles, package ecosystems, pinned TypeScript
  derivation, dialect module declarations, and dialect file placements are
  typed enums/associated types. Groups prove exact membership, role
  consistency, and deterministic ordering.
- Metadata, documentation, assets, and compiler-derived JavaScript are
  structural non-source artifact variants. JavaScript has no independently
  lowered executable body; source files contain only typed AST items.
- Helper catalogue entries contain ordered typed AST items, placement,
  visibility, and provenance. Helper references are discovered by traversing
  those items with the same verifier used for generated declarations; no
  helper carries a manual dependency/import list.
- The linker computes one deterministic transitive helper closure, emits every
  selected helper exactly once into its uniquely matching runtime file, and
  rejects missing, duplicate, cyclic, public, empty, or illegally placed
  helpers. Optional helpers and their exclusive dependencies remain absent.
- Generated declaration ownership and cross-file edges are reference-derived.
  The verifier rejects forged edges, private public-API dependencies,
  public-API-to-implementation edges, runtime-to-user edges, production-to-test
  edges, duplicate/unplaced declarations, and cycles unless a typed dialect
  policy explicitly permits the exact cycle.
- The dialect file-verification hook uses typed modules/placements and has
  proofs for Java's one-public-type rule plus C/C++ declaration,
  implementation, header, and complete-type placement values.
- In the Linux development container, both named milestone gates passed
  uncached. Rustfmt, Clippy, Buildifier, the typed-generation source policy,
  and repository source policy passed; the complete tracked-scope graph passed
  uncached, 264 of 264 tests. The frozen untracked M34-03 `stdlib-abs`
  package was the only excluded Bazel package and was not modified.
