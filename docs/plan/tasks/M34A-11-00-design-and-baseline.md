# M34A-11-00 — Specify C17 layers and migration baseline

- Status: complete
- Depends on: M34A-10

## Goal

Capture the language-specific architecture before implementing new production C generation.

## Definition of done

- Write C-specific AST, ownership/ABI, interface, catalogue/file, mapping-certificate, rendering/resource, module and proof specifications.
- Preserve existing target identity org.polyrust.c; distinguish C17 language version and the explicitly superseded mutable public ABI.
- Define the full 42-capability migration and an honest legacy/new-route boundary; no unsupported slot advertises Supports.
- List independently reviewable implementation tasks, each with exit evidence. Keep C compliance Fail until full cutover and review.
- Record baseline local gates and commit/push the specification checkpoint; leave unrelated stdlib-abs untouched.

## Tests and proof

- Run //tools/docs:documentation_test and //:buildifier_test in the Linux container.
- Run the complete tracked Bazel rule graph and //:release_gate with normal caching; retain native C and sanitizer results.
- Review C against shared phase/capability/certificate contracts and the Java module structure; no placeholder implementation counts as proof.

Test targets required by this slice must be added before it closes; proposed
future targets are not evidence of an existing implementation. All builds and
tests run in the Linux development container with pinned toolchains and normal
Bazel action/test caching.

## Commit gate

Record exact commands, counts, failures and dispositions here. Commit and push
this completed checkpoint with M34A-11-00 in the message. Keep the overall
C migration open until M34A-11-09; never substitute a partial slice for Pass.

## Baseline evidence (2026-09-08)

- `b195fc9e-8fd1-456a-b003-f81238962b7b`: all 435 tracked rules build and
  all 310 tests pass, including documentation, Buildifier, Rustfmt, Clippy,
  legacy C native/public consumers and sanitizers.
- `6897f2b8-0c31-4244-820d-02931f1dfab4`: all 247 release tests pass.
- `d003314d-3df7-4d2e-9756-3ad5af853ba9`: 50 cases and one portable test,
  evaluator/eight-target agreement and deterministic repeated manifests.

All runs use the existing Linux development container and pinned toolchains,
with normal action/test caching. Only untouched untracked stdlib-abs is excluded.
Eight C layer documents and ten checkpoint task files are present. This is a
specification/baseline completion, not C typed-backend compliance.
