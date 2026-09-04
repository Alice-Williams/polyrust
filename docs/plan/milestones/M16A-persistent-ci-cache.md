# M16A — Persist Bazel caches across GitHub Actions runs

- Status: complete
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

## Exit evidence

- Implementation checkpoint `9f257aa891e4db46dce50fa7fb75ead85c4b6ae3`
  passed hosted population run `33874194596`; the save action reported the
  expected immutable `v3` run-attempt key.
- Evidence checkpoint `034d9b5d01a6d1be70c397a0e66125865f61d45a`
  passed hosted restoration run `33877881376` in all eight jobs. GitHub
  restored the exact population key and then saved a refreshed key.
- The restored archive was 2,594 MB. The independent cold gate passed in
  14m55s and the complete persistent warm gate passed in 1m48s.
- Local actionlint 1.7.12, cache policy, Buildifier, documentation, and the
  uncached 237/237 tracked release suite passed before the implementation
  push.
