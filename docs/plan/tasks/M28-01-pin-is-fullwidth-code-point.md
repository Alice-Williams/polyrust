# M28-01 — Pin is-fullwidth-code-point 3.0.0

- Status: complete

## Goal

Retain the exact admitted upstream evidence without network access at test time.

## Definition of done

- Commit `80e5e314d86e5f76bd1b0573aa9d33e615a372db` is recorded.
- Implementation, declaration, type test, runtime test, package metadata,
  README, and MIT license are retained.
- An offline Bazel test verifies every Git blob hash.

## Tests

- `bazel test //third_party/is-fullwidth-code-point:provenance_test`
