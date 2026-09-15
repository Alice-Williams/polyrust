# M35-02B-03M — Conditional partial record initialization and moves

- Status: in-progress
- Parent: [M35-02B-03](M35-02B-03-structured-owned-places.md)
- Depends on: M35-02B-03L

## Contract

Discharge G's remaining conditional partial initialization/move requirement.
Observe pinned source/HIR/PostCleanup before choosing the closed admission
grammar. Preserve actual Boolean producer, branch-local field initialization or
move, live/dead field obligations and canonical exits on every normal path.
Flat record evidence plus scalar selection is not sufficient by composition.

## Ordered checkpoints

1. [M-01 — Compiler observations](M35-02B-03M-01-conditional-observations.md):
   complete; asserts both outcomes for conditional initialization, scoped field movement
   and early return versus continuation before admitting new body evidence.
2. [M-02 — Typed source frame](M35-02B-03M-02-conditional-source-frame.md):
   freeze the observed source forms, canonical control/field/scope identities
   and executable operation bindings behind private compiler-session inputs.
3. [M-03 — Complete correspondence](M35-02B-03M-03-conditional-correspondence.md):
   account for every normal path, initialization state, field movement, cleanup
   decision and exit with ordinary consumers and non-vacuous corruption tests.

## Definition of done and tests

- Specify concrete positive initialization and partial-move forms, source
  limits and compiler path budgets before implementing safe input construction.
- Each branch/merge authenticates field initialization state and ownership;
  emitted cleanup cannot read/drop an uninitialized or already moved field.
- Retain actual compiler drop-flag decisions and full typed nested/flat paths
  where admitted, rather than implementing a new source borrow checker.
- Both outcomes, early cleanup/continuation and failed-source controls prove
  exact field/owner/guard/drop correspondence. Substituted guards, field state,
  missing/duplicate initialization, moved-source reuse and cleanup mutations fail.
- Use private compiler-session evidence and executable typed mappings, with
  ordinary consumer projections and exact compile-negative boundary tests.
- Split observation, capability and complete-body work into focused child tasks;
  each requires full isolated gates and independent review before commit/push.
  C/Java heap output and native runtime proof remain separate.
