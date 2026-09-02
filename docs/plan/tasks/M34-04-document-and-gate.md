# M34-04 — Document and release-gate M34

- Status: planned

## Goal

Record the exact IEEE semantics, compositional backend audit, and proof
evidence, then prove M34 does not regress any completed port.

## Definition of done

- The milestone, task set, compatibility ledger, real-world port report,
  architecture, checker, evaluator, and portable-language documents reflect
  the implemented semantics.
- M34 participates in Buildifier, Rustfmt, Clippy, full-repository, and release
  gates.
- Every earlier native/differential port and every M30 source-policy invariant
  remains green.
- M34 is committed and pushed, and hosted GitHub CI passes before another
  repository is selected.

## Tests

- `bazel test //... --nocache_test_results --test_output=errors`
- `bazel test //:release_gate --nocache_test_results --test_output=errors`
