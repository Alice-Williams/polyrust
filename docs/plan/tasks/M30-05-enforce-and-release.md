# M30-05 — Enforce the import boundary and release

- Status: in-progress

## Goal

Make compositional dependency ownership a permanent checked invariant.

## Definition of done

- A Bazel policy test rejects generated import/include/use directives in body
  templates outside renderer implementations.
- Handwritten target consumer fixtures are explicitly scoped exceptions.
- Architecture, backend-author, and extension-example documentation use the
  fragment/helper-graph model.
- Backend contract tests cover dependency completeness and minimality.
- All M30 changes are committed, pushed, and hosted CI is green.

## Tests

- Source-policy and compile-fail API tests.
- `bazel test //... --nocache_test_results --test_output=errors`
- `bazel test //:release_gate --nocache_test_results --test_output=errors`

## Local release-candidate evidence

- The source-policy positive suite and deliberate Rust/template injection suite
  pass. The compile-fail doctests prove that raw text roles cannot construct
  source files and source roles cannot construct raw text files.
- All shared-codegen, eight-output backend, external-backend, Rustfmt, Clippy,
  and Buildifier focused gates pass 8/8 targets.
- The complete uncached repository gate passes 201/201 tests.
- The dedicated uncached release gate passes 178/178 tests.
- The task remains `in-progress` until this candidate is pushed and its hosted
  workflow is green, as required by the definition of done.
