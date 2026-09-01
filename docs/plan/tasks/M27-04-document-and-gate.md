# M27-04 — Document and release-gate M27

- Status: complete

## Goal

Record the semantic boundaries and reusable backend gaps closed by parse-ms,
then prove that M27 does not regress any earlier compatibility port.

## Definition of done

- The milestone, port report, compatibility ledger, checker reference, and
  C/C++ status reflect the implementation.
- parse-ms participates in Rustfmt, Clippy, Buildifier, full-repository, and
  release gates.
- Every earlier native/differential port remains green.
- The M27 commit is pushed and hosted GitHub CI passes.

## Tests

- `bazelisk test //... --test_output=errors` in the Linux container.
- `bazelisk test //:release_gate --test_output=errors` in the Linux container.
- Confirm Buildifier, Rustfmt, Clippy, target-native linters, sanitizers, and
  deterministic generation execute within those gates.

## Completion evidence

- The uncached full-repository gate passes 168/168 tests.
- The uncached release gate passes 146/146 tests.
- Both gates execute Buildifier, Rustfmt, Clippy, all earlier real-world ports,
  the eight parse-ms native packages, deterministic generation, provenance,
  public consumers, and C/C++ sanitizer checks.
