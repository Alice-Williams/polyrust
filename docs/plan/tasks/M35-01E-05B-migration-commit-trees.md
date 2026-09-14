# M35-01E-05B — Isolate and verify migration commit trees

- Status: in-progress
- Parent: [M35-01E-05](M35-01E-05-java-native-proof.md)
- Depends on: M35-01E-05A

## Implementation contract

- Inventory the existing dirty/untracked tree before staging. Preserve unrelated
  M34 child-ownership work and unrelated examples byte-for-byte; do not broadly
  stage, reset or delete them. Preserve any preexisting staged changes.
- Prepare separate C compiler-front-end and Java retrofit checkpoints. Use an
  isolated worktree/index and explicit reviewed paths; do not edit the user's
  checkout to manufacture an earlier checkpoint.
- Shared files and build declarations must match each proposed checkpoint.
  The C test-module list needs a snapshot-only split of unrelated child-graph
  registrations; the original pending tests and implementation stay untouched.
- Archive exact proposed Git trees into isolated Linux test directories. Use
  the pinned DevContainer toolchains and authoritative Bazel targets there,
  retaining dependency/action caches without relying on unstaged host files.
- Include required source/build/docs files, but exclude outputs, caches,
  bytecode, images and credentials. Confirm archive contents and Git tree IDs.

## Definition of done and tests

- The proposed migration tip passes the complete release/frontend/C/Java/shared/
  lint/docs gate from its own exact source tree, with no disabled tests.
- The separate C checkpoint passes its matching authoritative gates with
  historical Java retained; Java's checkpoint adds its implementation/proof.
- Required files are not supplied only by ignored/untracked working-tree state.
  Review the actual proposed diff and independently verify exclusion boundaries.
- Record exact tree IDs and gate evidence. No push until all required checks pass.

## Boundary

This task does not authorize committing unfinished unrelated M34 work. E05C
performs final commits/push after the verified trees are ready.

## Isolation evidence in progress

The normal user index remains untouched. Alternate indexes explicitly selected
656 migration paths and excluded 27 unrelated/artifact paths. The overlapping
C module list is split only in the snapshots; the original child-graph work
remains intact. The C snapshot retains historical Java; Java-specific frontend
files and declarations are introduced in the subsequent snapshot.

An initial archive attempt inherited host Git CRLF conversion and was stopped
after Linux script/format failures (`f3af1d29-53ea-429b-8da5-0ac1b02b5261`).
This was an archive transport defect, not a passing or completed source gate.
Archive creation now explicitly disables conversion; every payload's Git blob
hash and executable mode is checked against `git ls-tree` before extraction.
No tests were disabled. Exact final trees and successful gates remain pending.
