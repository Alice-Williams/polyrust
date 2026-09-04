# M16A — Persist Bazel caches across GitHub Actions runs

- Status: in-progress
- Phase: 5
- Depends on: M16

## Outcome

Make GitHub Actions restore and refresh Bazel caches across workflow runs while
retaining a genuinely isolated cache-cold release proof.

## Task sequence

1. [M16A-01 — separate cold proof from persistent warm caches](../tasks/M16A-01-persistent-bazel-cache.md)

## Required exit evidence

- The cache contract in `docs/specification/ci-cache.md` is enforced by a
  tracked Bazel test.
- Local workflow lint, policy, documentation, and release gates pass.
- A first hosted run populates a new compatible cache lineage.
- A later hosted run restores that lineage and both its cold and warm release
  gates pass.
- The completed checkpoint is committed and pushed.

## Scope boundary

This milestone does not provision an external Bazel remote-cache service,
cache checkout contents, or skip semantic work based on cache state.

