# M32-05 — Document and release-gate M32

- Status: complete

## Goal

Record provenance, version boundaries, reusable string semantics, and proof
evidence, then prove M32 does not regress any completed port.

## Completion evidence

- The strict architecture, checker, evaluator, portable-language,
  compatibility, ABI, port, milestone, and task documents describe the
  implemented boundary.
- The port suite passes 17/17 targets; the uncached repository and release
  gates pass 233/233 and 210/210 tests respectively.
- Every M30 source-policy invariant and every M31/earlier compatibility port
  remains green. Hosted workflow
  [33592407791](https://github.com/Alice-Williams/polyrust/actions/runs/33592407791)
  passes `84a81eb92f54cfbc37a4dd6013bee036c14d4939` across both determinism
  hosts, cross-host comparison, and cache-cold/cache-warm complete gates.

## Definition of done

- The milestone, task set, compatibility ledger, real-world port report,
  checker, evaluator, portable language map, and backend compliance audit
  reflect the implemented semantics.
- M32 participates in Buildifier, Rustfmt, Clippy, full-repository, and release
  gates.
- Every earlier native/differential port remains green.
- M32 is committed and pushed, and hosted GitHub CI passes before another
  repository is selected.

## Tests

- `bazel test //... --nocache_test_results --test_output=errors`
- `bazel test //:release_gate --nocache_test_results --test_output=errors`
