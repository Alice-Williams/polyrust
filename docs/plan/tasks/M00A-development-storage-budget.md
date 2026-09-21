# M00A — Bound local development storage

- Status: in-progress
- Depends on: M00

## Outcome

Bound disposable local caches and protect the host's shared development drive.
Budgets are decimal GB: 50 GB artifact cache, 200 GB total Docker target,
150 GB minimum host free space. This must not disable cached test results.

## Implementation checklist

- [x] Align future Dev Container volume mounts with actual Bazel cache paths.
- [x] Evict oldest disposable AC/CAS entries before/after local batch builds.
- [x] Check host free space and Docker filesystem usage before/during builds.
- [x] Verify focused Bazel tests, formatting and cache reuse in the container.
- [x] Trim existing cache and record measured usage.
- [ ] Establish a safe hard disk-capacity limit, or record a precise blocker.

## Required evidence

- Synthetic tests cover size/age eviction, preserved action results, symlink
  and unexpected-file rejection, low-space rejection, and process cleanup.
- Normal Bazelisk invocations automatically invoke the guard; Linux CI remains
  independent of this workstation's storage limits.
- Existing source changes, active output bases and Docker images remain intact.
- A build-time guard must not be reported as a hard Docker disk quota.

## Scope boundary

Do not reset Docker, delete containers/volumes, prune arbitrary temporary source
trees, or truncate a virtual disk containing a larger filesystem. The Docker
data disk is shared with the capnp experiment; all its persistent state matters.

## Evidence — 2026-09-21

- Container Bazel passed `//tools/dev:storage_budget_test` (15 safety tests),
  `//tools/ci:cache_policy_test`, `//tools/docs:documentation_test`, and
  `//:buildifier_test`. Ruff check and format check pass for the new Python files.
- Normal Bazelisk invocation selected `tools/bazel` without extra flags or an
  opt-in environment variable. Existing test-result caching remains enabled.
- Initial maintenance removed 76.21 GB of disposable cached artifacts and
  retained 49.98 GB; Docker's live filesystem usage is about 178.6 GB.
- Source files, active Bazel output bases, containers and images were preserved.
- The host VHDX still occupies about 280 GB; cache deletion is not compaction.

## Remaining hard-cap blocker

The existing Docker-managed WSL data disk still has approximately 1 TiB virtual
capacity. Docker documents its disk-limit control for Hyper-V, not this WSL
backend. `wsl --manage docker-desktop --resize` would target the distribution's
different disk, not `G:\Docker\wsl\disk\docker_data.vhdx`.

No unsupported configuration key, filesystem shrink, or VHD truncation was
attempted. A hard 200 GB disk limit needs separately planned offline maintenance
with a validated backup and restoration of both projects' Docker state. Until
then, the implemented thresholds only control wrapped polyrust builds.
