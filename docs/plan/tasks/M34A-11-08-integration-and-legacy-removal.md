# M34A-11-08 — Integrate C generation and delete legacy fragments

- Status: planned
- Depends on: M34A-11-07

## Goal

Atomically replace CBackend's legacy path with the checked CoreIR/AST pipeline.

## Definition of done

- Route both typed and CheckedProgram compatibility entry points through the shared compiler adapter; typed entry exposes only resource-capacity failures.
- Preserve org.polyrust.c and package entry paths; update ABI consumers deliberately for opaque ownership without weakening behavior assertions.
- Delete CCode, generator.rs, raw runtime.c/runtime.h templates and manual import/helper attachment; no successful fallback remains.
- Generate actual reviewable C examples under a documented curated-snapshot exception and require exact regeneration.
- Update CLI/conformance/build/policy integration and C ABI/spec/coverage ledger with honest proof status.

## Tests and proof

- Every historical C port, separate public consumer, native negative, style/strict-compiler and sanitizer gate.
- Three-generation byte equality, AST determinism, evaluator and all eight targets agree; no regressions in Java.
- Source-policy tests prove all legacy routes and raw source disappear; readonly-runfiles/native copy behavior remains valid.
- All tracked rules and release tests plus supplementary Linux Cargo compatibility and hosted CI before checkpoint review.

Test targets required by this slice must be added before it closes; proposed
future targets are not evidence of an existing implementation. All builds and
tests run in the Linux development container with pinned toolchains and normal
Bazel action/test caching.

## Commit gate

Record exact commands, counts, failures and dispositions here. Commit and push
this completed checkpoint with M34A-11-08 in the message. Keep the overall
C migration open until M34A-11-09; never substitute a partial slice for Pass.
