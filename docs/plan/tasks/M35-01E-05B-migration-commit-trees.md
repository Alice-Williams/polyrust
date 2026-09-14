# M35-01E-05B — Isolate and verify migration commit trees

- Status: complete
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

## Isolation evidence

The normal user index was untouched during preparation. Alternate indexes explicitly selected
656 migration paths and excluded 27 unrelated/artifact paths. The overlapping
C module list is split only in the snapshots; the original child-graph work
remains intact. The C snapshot retains historical Java; Java-specific frontend
files and declarations are introduced in the subsequent snapshot.

An initial archive attempt inherited host Git CRLF conversion and was stopped
after Linux script/format failures (`f3af1d29-53ea-429b-8da5-0ac1b02b5261`).
This was an archive transport defect, not a passing or completed source gate.
Archive creation now explicitly disables conversion; every payload's Git blob
hash and executable mode is checked against `git ls-tree` before extraction.
No tests were disabled.

The first C tree exposed an actual checkpoint-order dependency: historical
Java lacked the shared linker's new dependency-associated types and catalogue
field (`0a05bb72-d249-4416-9a0f-b9e5a1df4cf9`, E0046/E0063). The C checkpoint
now uses `Infallible` dependency witnesses, an exhaustive empty-match hook and
an empty dependency catalogue. This permits no new Java call and leaves its
historical tests enabled. The following Java checkpoint replaces those types
with its real opaque dependency API.

## Exact tested trees

| Checkpoint | Git tree | Full Linux/Bazel gate | Result |
| --- | --- | --- | --- |
| C | `0ba18d4c14366991f3a000301688b301d9c58243` | `2324d014-af9e-4a1c-8e35-27b9c2536662` | 346/346 tests, 419 targets |
| Java | `613bdf2452731831b178783082c44448bbbaa6aa` | `f4b809dc-19e0-476f-b7fe-8c96d1b4c63d` | 373/373 tests, 487 targets |

Archives contained 1,956 and 2,080 files respectively. Every payload and
executable mode matched its Git blob. Fresh complete source directories were
swapped only after each gate exited; the same isolated workspace path preserved
valid Bazel action/test caches without overlaying files from another tree.

Both gates used the same explicit target set:

```sh
bazelisk --batch test //:release_gate //experiments/rustc-frontend:all \
  //crates/backend-c:portable_backend_c_test \
  //crates/codegen:portable_codegen_test \
  //crates/codegen:typed_pipeline_compile_fail_test \
  //crates/backend-java:portable_backend_java_test \
  //crates/backend-java:java_typed_compile_fail_test \
  //tools/docs:documentation_test --keep_going
```

Fresh Sol Extra High `migration_candidate_tree_review` confirmed the source,
module/build, mode and exclusion boundaries. After the sequencing repair,
`c_checkpoint_compatibility_review` independently approved the exact final C
tree with no core or optional findings. The Java tree was unchanged.

These identities describe the tested implementation checkpoints. This completion
record is a later documentation-only update, not part of those earlier trees.
