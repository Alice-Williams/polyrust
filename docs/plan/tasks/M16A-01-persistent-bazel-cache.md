# M16A-01 — Separate cold proof from persistent warm caches

- Status: in-progress
- Depends on: M16
- Blocks: M16A completion

## Goal

Correct the release workflow so its restored Bazel caches accelerate later
GitHub Actions runs instead of being deleted by the cache-cold step.

## Definition of done

- Cache restore and save use an immutable per-run key plus a toolchain- and
  dependency-compatible restore prefix.
- Persistent Bazelisk, repository, and disk caches are mounted only into the
  warm release gate.
- The cold release gate mounts a separate freshly created and asserted-empty
  Bazel cache tree.
- Cargo downloads and the pinned advisory tool remain safely reusable in both
  modes and no credential or checkout path is cached.
- Both gates execute the complete unchanged release script.
- A successful run can publish refreshed cache contents; failed runs cannot.

## Tests

- Add a Bazel policy test which fails when cache key rotation, compatibility
  inputs, save-after-gates ordering, cold/warm path separation, or the empty
  cold assertion is removed.
- Run Buildifier and the documentation test.
- Run actionlint against `.github/workflows/ci.yml`.
- Run the complete release gate in the Linux development container.
- Push a population run, then push a documentation-evidence checkpoint and
  prove its workflow restored the new cache lineage and passed cold and warm
  gates.

## Commit gate

Commit and push this specification checkpoint before implementation. Commit
and push the workflow/policy-test checkpoint only after all local gates pass.
Mark complete only after the second hosted run supplies restoration evidence.

## Exit evidence

Implementation is complete locally pending the two hosted cache-lineage runs:

- The workflow restores and saves versioned persistent caches with a unique
  run-attempt key and a toolchain/dependency-compatible restore prefix.
- Cache-cold mounts a freshly recreated, asserted-empty `polyrust-cold` Bazel
  tree; cache-warm alone mounts the restored `polyrust-cache` Bazel tree.
- `//tools/ci:cache_policy_test` is part of the explicit release suite and
  checks rotation, path separation, ordering, empty-cold proof, and complete
  release-script execution in both modes.
- The focused cache-policy, Buildifier, and documentation tests pass.
  Actionlint 1.7.12 accepts the workflow.
- The uncached authoritative tracked release gate passes 237 of 237 tests.
  The broader local `//...` invocation is presently blocked only by the
  unrelated untracked `examples/real-world/stdlib-abs` worktree, which refers
  to its not-yet-created `differential_test.sh`; that user-owned work remains
  untouched and is absent from clean hosted checkouts.
