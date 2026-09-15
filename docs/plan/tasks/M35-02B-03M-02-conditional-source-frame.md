# M35-02B-03M-02 — Typed conditional record source frame

- Status: planned
- Parent: [M35-02B-03M](M35-02B-03M-conditional-owned-records.md)
- Depends on: M35-02B-03M-01

## Contract

Freeze the closed source grammar established by M-01 and introduce private,
compiler-session source evidence for its conditional initialization and
partial-move forms. Use a closed enum for the admitted source forms, actual
Boolean parameter HirId, actual branch/block identities, canonical exits and
nominal field paths. Do not use string tags or infer control flow from counts.

Retain the existing executable Box-construction and record-construction
capability bindings. This source frame composes authenticated operations; it
does not need a duplicate capability for each spelling of an if statement.
Any additional binding genuinely required by the observations must receive
its own typed input/output contract before implementation.

## Definition of done and tests

- Every source form is described normatively before its reader is enabled.
- Inputs can only be constructed from canonical compiler-owned source after
  successful analysis; arbitrary expressions/owners and unsupported forms fail.
- Missing initialization is represented explicitly, never as a fake value.
- Branch-local ownership transfers retain actual field identity and scope;
  shadowing cannot alias different source bindings.
- Existing Supports mappings consume every required checked operation with
  exact associated input/context/output types in ordinary consumer tests.
- Compile-negative tests protect the new source evidence and prevent its use
  as complete body evidence. Source-only success does not admit target output.
- Positive/negative source fixtures, exact boundary tests, full isolated gate,
  fresh broad review and final exact-tree gate precede commit/push.
