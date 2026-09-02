# M33-04 — Document and release-gate M33

- Status: in progress

## Goal

Record the typed admission boundary, exact IEEE semantics, compositional
backend audit, and proof evidence, then prove M33 does not regress any completed
port.

## Definition of done

- The milestone, task set, compatibility ledger, real-world port report,
  architecture, checker, evaluator, and portable-language documents reflect the
  implemented semantics.
- M33 participates in Buildifier, Rustfmt, Clippy, full-repository, and release
  gates.
- Every earlier native/differential port and every M30 source-policy invariant
  remains green.
- M33 is committed and pushed, and hosted GitHub CI passes before another
  repository is selected.

## Tests

- `bazel test //... --nocache_test_results --test_output=errors`
- `bazel test //:release_gate --nocache_test_results --test_output=errors`

## Local completion evidence

- The strict architecture, semantic references, task set, compatibility
  ledger, and dedicated port report describe the implemented admission and
  dependency-ownership boundaries.
- M33 is registered in Buildifier, Rustfmt, Clippy, repository-wide, and
  release targets.
- The uncached repository-wide run passes 250/250 tests and the uncached
  release gate passes 227/227 tests in the Linux development container.
- Hosted CI remains the final closure condition.
