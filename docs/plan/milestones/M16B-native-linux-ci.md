# M16B — Native Linux CI

- Status: in-progress
- Phase: 5
- Depends on: M16A

## Outcome

Remove development-container construction from GitHub Actions and execute the
same cached Bazel release contract directly on Linux runners.

## Task sequence

1. [M16B-01 — migrate CI to native Linux execution](../tasks/M16B-01-native-linux-ci.md)

## Required exit evidence

- The native Linux specification is enforced by a tracked Bazel policy test.
- No Linux workflow step builds or runs Docker.
- Native bootstrap tools and cache compatibility inputs are pinned.
- Local lint, documentation, policy, and release targets pass.
- A hosted run passes all jobs and saves the refreshed native cache lineage.
- The completed checkpoint is committed and pushed.

## Scope boundary

This milestone does not vendor external repositories, remove the Windows
development-container contract, or redesign language backends.
