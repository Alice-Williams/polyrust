# M34A-11-09 — Complete C adversarial proof and fresh review loop

- Status: planned
- Depends on: M34A-11-08

## Goal

Close C only after integrated native evidence and an independently evaluated clean review.

## Definition of done

- Expand structured fuzz/mutation grammar, ownership and operation corpora with explicit seeds/categories and coverage inventory.
- Prove strict compiler acceptance, public ABI behavior, fault-injection cleanup and sanitizer behavior on every admitted feature.
- Run full tracked/release/eight-target determinism gates with caches enabled, commit/push and check the exact hosted SHA.
- Run a fresh uncapped Sol Extra High read-only review; reproduce/repair accepted core defects and repeat from a new immutable commit.
- Record justified rejections and optional deferred extensions; mark C Pass only with no unresolved core errors, then request the user's review.

## Tests and proof

- All required tests in the C proof specification, including no-vacuity compiler negatives and allocation-failure oracle controls.
- Fresh immutable review spanning every layer, not an arbitrary maximum issue count.
- Final complete local gates and exact-commit hosted CI; Git remote matches the checkpoint.
- No remaining legacy C paths, unwarranted Supports claims, oversized new modules or skipped historical targets.

Test targets required by this slice must be added before it closes; proposed
future targets are not evidence of an existing implementation. All builds and
tests run in the Linux development container with pinned toolchains and normal
Bazel action/test caching.

## Commit gate

Record exact commands, counts, failures and dispositions here. Commit and push
this completed checkpoint with M34A-11-09 in the message. Keep the overall
C migration open until M34A-11-09; never substitute a partial slice for Pass.
