# M34A-11-02 — Implement C AST, scope and ownership verification

- Status: planned
- Depends on: M34A-11-01R, M34A-11-00R, M34A-10AB

## Goal

Make local grammar, symbol and safety obligations explicit before a C package can become verified.

## Implementation slices

This milestone is intentionally split into independently tested checkpoints;
it closes only when all four do. Each new source module remains focused.

1. [02A — Registries and nominal identities](M34A-11-02A-registries-and-nominals.md)
2. [02B — Expressions, declarations and files](M34A-11-02B-expressions-and-declarations.md)
3. [02C — Contextual scope and control flow](M34A-11-02C-scope-and-flow.md)
4. [02D — Ownership, bounds and arithmetic proof](M34A-11-02D-ownership-and-range-proof.md)

## Definition of done

- Add typed expressions, places, statements, initializers, declarations/definitions, nominal/member/function registrations and file-role nodes.
- Check exact call/member types, namespaces, complete types, prototype consistency, initialization, returns, labels/cases and control-flow joins.
- Track ownership identities, allocator provenance, initialized prefixes, borrows and Empty/Live/Moved/Dropped states on every exit.
- Require derived bounds/range/tag facts for dangerous arithmetic, shifts, dereferences and union access; reject forged safety metadata.
- Keep unknown target input fallible and verified state private. No text scanner, syntax escape or manual certificate constructor.

## Tests and proof

- Positive and rejected mutation per AST category and contextual rule; cross-registry/owner/signature/reference forgery tests.
- Ownership tests for use after move/drop, borrow escape, wrong allocator, partial initialization, cleanup skipped by return/goto and branch joins.
- Arithmetic tests for signed extrema, zero division, invalid shifts, overflowed allocation sizes and bounds checked after pointer formation.
- Focused Rust/compile-fail and complete cached tracked/release gates. Native compiler oracles expand in M34A-11-04.

Test targets required by this slice must be added before it closes; proposed
future targets are not evidence of an existing implementation. All builds and
tests run in the Linux development container with pinned toolchains and normal
Bazel action/test caching.

## Commit gate

Record exact commands, counts, failures and dispositions here. Commit and push
this completed checkpoint with M34A-11-02 in the message. Keep the overall
C migration open until M34A-11-09; never substitute a partial slice for Pass.
