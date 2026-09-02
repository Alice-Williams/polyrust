# M30-05 — Enforce the import boundary and release

- Status: complete

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

## Release evidence

- The source-policy positive suite and deliberate Rust/template injection suite
  pass. The compile-fail doctests prove that raw text roles cannot construct
  source files and source roles cannot construct raw text files.
- All shared-codegen, eight-output backend, external-backend, Rustfmt, Clippy,
  and Buildifier focused gates pass 8/8 targets.
- The complete uncached repository gate passes 201/201 tests.
- The dedicated uncached release gate passes 178/178 tests.
- Hosted workflow
  [33577166696](https://github.com/Alice-Williams/polyrust/actions/runs/33577166696)
  passes at `64cec7defbad6b61c56511fc5a986fdb1b08ecf2`: Windows contract,
  Rust 1.98.0 and stable, fast checks, two-host deterministic manifests,
  cross-host byte comparison, and cache-cold/cache-warm release gates are all
  green.
- The first clean candidate run exposed that the new shell policy runner lacked
  its tracked executable bit. Commit `64cec7d` records mode `100755`; the green
  clean-checkout gate proves the Windows bind mount can no longer mask that
  failure.
