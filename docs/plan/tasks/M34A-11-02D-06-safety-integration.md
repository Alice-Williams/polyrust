# M34A-11-02D-06 — Compose and review the C safety verifier

- Status: planned
- Depends on: M34A-11-02D-05

## Goal

Close all 02D obligations together before shared linking/certification begins.

## Definition of done

- Compose local reconstruction, inventories, lexical/initialization checks,
  constants/layouts, sequencing, ranges/progress, ownership and callable proofs.
- Return only the private immutable state needed by the later shared verifier;
  no public verify-only renderer or conversion from legacy source exists.
- Recheck mutually dependent obligations against the same actual registry/AST.
  Deleting projected evidence cannot erase registrations or proof obligations.
- Audit every 02D checklist row and all closed AST variants against positive,
  rejected mutation and compile-fail evidence; no pending requirement becomes
  an optional feature merely because it is difficult.
- Obtain fresh uncapped Sol Extra High review; evaluate every finding, repair
  accepted defects and repeat with a new reviewer until no core errors remain.
  Record explicit reasons for rejected findings and separate optional expansion.

## Tests and proof

- Combined type-valid but unsafe programs spanning guards, aliasing, allocator
  identity, call effects, active union, loop progress, partial output and cleanup.
- Private-evidence/API compile-fail and mutation controls; no unchecked safe API
  can construct a verified/shared/render-ready package.
- Full tracked Bazel rule graph, release gate, Rust/Bazel linters and all eight
  targets' deterministic conformance, using normal caches in the Linux container.

## Commit gate

Record exact evidence and review dispositions, close 02D and its parent 02 only
when all exit criteria pass, then commit/push M34A-11-02D-06. C compliance remains
open until the renderer, all capabilities, historical cutover and stage 09 pass.
