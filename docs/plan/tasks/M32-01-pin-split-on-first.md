# M32-01 — Pin split-on-first 3.0.0

- Status: complete

## Completion evidence

- All seven retained files hash to the Git blob IDs at the pinned v3.0.0
  commit.
- The offline provenance test, documentation link test, and repository
  Buildifier gate pass in the Linux development container.
- The milestone fixes the complete v3 string-only admission contract and
  records both the declaration/runtime result mismatch and the v4 regex
  version boundary before implementation begins.

## Goal

Retain the exact complete typed upstream evidence without network access at
test time, and freeze the semantic admission boundary before implementation.

## Definition of done

- Tag `v3.0.0`, annotated tag object
  `3f4a69e8e1715e4e60060d2f04ccc68a1305c96f`, and commit
  `d6bf86163df4e6490b134c303477644a52736997` are recorded.
- Implementation, declaration, declaration test, runtime test, package
  metadata, README, and MIT license are retained byte-for-byte.
- The exact `String × String` input domain, empty-list result, well-formed
  Unicode boundary, and later v4 `RegExp` non-goal are recorded.
- An offline Bazel test verifies every upstream Git blob ID.

## Tests

- `bazel test //third_party/split-on-first:provenance_test`
