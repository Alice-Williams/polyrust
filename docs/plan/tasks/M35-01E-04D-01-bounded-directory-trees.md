# M35-01E-04D-01 — Reusable bounded atomic directory staging

- Status: complete
- Parent: [M35-01E-04D](M35-01E-04D-java-bundle-publication.md)
- Depends on: M35-01E-04C

## Implementation contract

- Separate the existing language-neutral staging/no-replace transaction from C
  payload-count wrappers. Reuse the pinned Linux directory publisher; do not add
  another subprocess, shell publication implementation or third-party dependency.
- Select flat versus bounded nested relative paths through a typed policy.
  Preserve C's flat-name/count restrictions. Java's caller supplies its verified
  canonical source/manifest inventory and bounded directory-prefix inventory.
- Before staging, reject absolute paths, dot/parent components, duplicate names,
  file/directory prefix conflicts, path/depth/count overflow and total byte overflow.
- Create private transaction directories and files exclusively. Record each
  successful creation; on failure clean only those exact files and directories in
  reverse dependency order. Never recursively traverse a destination for cleanup.
- Preserve atomic refusal of existing and concurrently created destinations.

## Definition of done and tests

- Existing C publication/platform/Clippy tests pass unchanged in intent.
- Native nested-output tests cover exact and one-over path/depth/directory/count
  bounds, duplicate/prefix conflicts, traversal and absolute-path rejection.
- Partial writes, directory creation and helper failures leave no owned staging
  debris or published prefix. Existing and late file/directory/symlink destinations
  remain unchanged; concurrent publishers have exactly one complete winner.
- Rust/Bazel formatting and independently cached publisher targets pass.

## Verification evidence

- Shared `directory_publication` Rust library separates canonical path validation
  from private staging/commit. C retains its three-file and 3N+1 flat wrappers.
- Focused gate `12423ffc-99be-4995-8ec9-8634417c94a7`: 9/9 targets pass.
  Tests include real short writes using RLIMIT_FSIZE, partial directory/file
  creation failures and a synchronized two-publisher no-replace race.
- Full gate `af39ad72-68fb-48e3-9979-770a9f0a302c`: 367/367 tests across
  464 targets pass; no disabled tests.
- Fresh Sol Extra High read-only review `bounded_publisher_review` found no core
  defects or material proof gaps. Its optional locale-stability suggestion was
  accepted: the nested fault-test subprocess environment pins LC_ALL=C.
- The locale change is included in the later all-green 367-test gate
  `5cc1c416-2067-47d6-ac75-6e332bfbbae5`.
