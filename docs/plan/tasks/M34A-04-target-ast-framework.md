# M34A-04 — Establish the typed target-AST framework

- Status: complete
- Depends on: M34A-03
- Historical note: ADR-0005 replaces executable template IDs with closed
  grammar/format enums and adds the render-ready proof state in M34A-08R.

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

## Evidence

- The phase-level `TargetDialect` now has a `TypedAstDialect` extension whose
  unresolved associated type is compile-time constrained to
  `TargetAstPackage<Self>`. The package owns category-specific typed IDs,
  generated declarations, expression and statement arenas, files, groups,
  template enums, origins, invocation kinds, and provenance.
- Dialects own their expression, statement, file-item, primitive, constructed,
  known/runtime symbol, visibility, declaration-kind, and template enums.
  There is no universal target switch or universal source expression enum.
- `Expr<D, T>` makes known target expression types phantom-typed. Dynamic
  input-defined declarations remain typed IDs whose signatures and references
  are checked by the unresolved verifier. Compile-fail documentation proves
  both phantom-type and expression/statement category boundaries.
- The deliberately small test dialect verifies known, runtime, generated, and
  interface callables; receiver/invocation shape; expression postorder and
  types; all declaration/reference categories; file/group ownership; safe
  paths; provenance; stable diagnostics; and three identical canonical dumps.
- `//tools/policy:typed_generation_source_policy_test` rejects opaque
  executable variants, source-string/byte fields, document fields, and
  `String`/`Document` conversions into executable AST categories. Its embedded
  fault-injection cases prove every prohibition is live.
- Linux-container Bazel invocation
  `e55a183d-77ad-43ad-a1c2-7fb6748a463d` passed all codegen tests and doctests,
  the typed-generation source policy, Buildifier, Rustfmt, and Clippy: 12/12
  tests.
