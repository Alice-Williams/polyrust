# M29-01 — Pin normalize-newline 5.0.0

- Status: complete

## Goal

Retain the exact admitted upstream evidence without network access at test time.

## Definition of done

- Commit `bc6982d73ebd62de3729435d9baf8731ca274f7a` is recorded.
- Implementation, declaration, runtime test, package metadata, README, and MIT
  license are retained.
- The typed overload boundary and invalid-dynamic-input exclusion are recorded.
- An offline Bazel test verifies every Git blob hash.

## Tests

- `bazel test //third_party/normalize-newline:provenance_test`
