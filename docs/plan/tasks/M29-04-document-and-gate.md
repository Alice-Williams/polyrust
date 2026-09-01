# M29-04 — Document and release-gate M29

- Status: in-progress

## Goal

Record the overload/value boundary and reusable byte gap, then prove M29 does
not regress any earlier port.

## Definition of done

- The milestone, task set, compatibility ledger, port report, checker, and
  language map reflect the implementation.
- M29 participates in Rustfmt, Clippy, Buildifier, full-repository, and release
  gates.
- Every earlier native/differential port remains green.
- The M29 commit is pushed and hosted GitHub CI passes.

## Tests

- `bazel test //... --nocache_test_results --test_output=errors`
- `bazel test //:release_gate --nocache_test_results --test_output=errors`
