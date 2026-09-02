# M31-01 — Pin has-flag 5.0.1

- Status: planned

## Goal

Retain the exact admitted upstream evidence without network access at test
time.

## Definition of done

- Tag `v5.0.1` and commit
  `63fde682532a6e0bb155125d03a66989e0b0ce24` are recorded.
- Implementation, declaration, runtime tests, declaration tests, package
  metadata, README, and MIT license are retained byte-for-byte.
- The explicit-`argv`, host-default, and lone-surrogate boundaries are
  recorded.
- An offline Bazel test verifies every upstream Git blob ID.

## Tests

- `bazel test //third_party/has-flag:provenance_test`
