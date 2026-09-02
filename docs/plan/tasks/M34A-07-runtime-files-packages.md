# M34A-07 — Make runtimes, files, and packages structural

- Status: planned
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
