# M34A-04 — Establish the typed target-AST framework

- Status: planned
- Depends on: M34A-03

## Goal

Provide shared dialect/package infrastructure while ensuring each language
owns grammar-correct types.

## Definition of done

- `TargetDialect`, typed expression handles, generated symbol IDs, package/
  file/group IDs, provenance, and unresolved package infrastructure exist.
- Grammar categories are distinct Rust types; known operations can use
  phantom-typed expressions while input-defined declarations use verified IDs.
- Operators, invocation kinds, visibility, declaration kinds, symbol origins,
  and template IDs are enums.
- The framework has no universal source-expression/statement enum and no target
  switch.
- Production AST APIs have no raw/verbatim/snippet/token-string/executable-code
  node or conversion from `Document`/`String`.
- A deliberately small test dialect proves typed builders, dynamic declaration
  verification, traversal, deterministic dumps, and invalid-tree diagnostics.
- Source policy rejects newly introduced opaque executable fields outside
  renderer-private views.

## Tests

- `bazel test //crates/codegen:target_ast_test --nocache_test_results --test_output=errors`
- `bazel test //crates/codegen:target_ast_compile_fail_test --nocache_test_results --test_output=errors`
- `bazel test //tools/policy:typed_generation_source_policy_test --nocache_test_results --test_output=errors`

## Commit gate

Commit and push `M34A-04: add typed target-AST framework` after focused tests
and `//crates/codegen:all` pass in the dev container.
