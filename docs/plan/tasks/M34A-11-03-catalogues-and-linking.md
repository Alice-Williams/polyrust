# M34A-11-03 — Implement C catalogues and derived header/source linking

- Status: planned
- Depends on: M34A-11-02

## Goal

Resolve every dependency, name and declaration placement from typed references.

## Definition of done

- Implement CDialect shared AST binding, exact standard/runtime signatures, typed known constants and source-file roles.
- Derive includes, guards, forward declarations and complete-definition order from a typed dependency graph, including runtime helpers.
- Allocate names with ordinary/tag/label/member namespaces, protected standard symbols and stable generated identities.
- Authenticate complete file/prototype/member/definition inventory in both directions after helper composition.
- Reject impossible by-value cycles, linkage/definition collisions, signature mismatches and leaked private layouts.

## Tests and proof

- Enforce the macro/name/typed Math dependency and public ABI identity-map
  contracts in c/platform-and-proof.md, including negative link and collision
  controls. Test-only stdio/fenv dependencies cannot leak into scalar packages.

- Exact include/placement matrices, idempotent duplicate references and unused-dependency elimination.
- Pointer-cycle positive / by-value-cycle negative; typedef/tag versus ordinary namespace collisions; same-signature wrong-owner references.
- Missing/extra/moved file and declaration inventory fault injection; repeated linking/dumps deterministic.
- Shared typed pipeline, linker, C Rust, policy and full tracked/release gates.

Test targets required by this slice must be added before it closes; proposed
future targets are not evidence of an existing implementation. All builds and
tests run in the Linux development container with pinned toolchains and normal
Bazel action/test caching.

## Commit gate

Record exact commands, counts, failures and dispositions here. Commit and push
this completed checkpoint with M34A-11-03 in the message. Keep the overall
C migration open until M34A-11-09; never substitute a partial slice for Pass.
