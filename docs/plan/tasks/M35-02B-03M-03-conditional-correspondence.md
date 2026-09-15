# M35-02B-03M-03 — Complete conditional record correspondence

- Status: planned
- Parent: [M35-02B-03M](M35-02B-03M-conditional-owned-records.md)
- Depends on: M35-02B-03M-02

## Contract

Certify the complete source-to-normal-MIR relation for every closed form from
M-02. Authenticate both outcomes and all compiler flag decisions controlling
initialization-sensitive and partial-move cleanup. Consume the source frame;
do not compose unrelated scalar and flat-record certificates by assumption.

## Definition of done and tests

- Exact source Boolean provenance selects each source arm; compiler drop flags
  are separately authenticated bookkeeping, never substituted for that guard.
- Constructor-rooted ownership is tracked across each aggregate, branch-local
  extraction and merge. Actual whole-record versus remaining-field cleanup is
  represented by typed events with full Places and Locations.
- Uninitialized and moved fields produce no read/drop obligation. Every live
  leaf is accounted for exactly once on each normal exit, with correct scopes.
- Every call, owning local, assignment, flag read/write, normal block, selected
  scalar read and Return is accounted for within documented budgets.
- Ordinary consumers assert complete operation/branch/cleanup/exit projections
  without accessing relation internals; canonical inputs reach real Supports
  binding functions rather than marker-only traits.
- Coherent wrong-guard, wrong-field state, duplicate initialization/movement,
  stale-owner, missing/double/out-of-order cleanup and flag mutations reject.
  Valid renaming or other claimed invariants have independent positive controls.
- Unsupported or invalid Rust publishes no proof; exact compile-negative tests
  prevent source-frame erasure and arbitrary-MIR injection.
- Full isolated historical/native/lint gate, fresh broad review and final
  exact-tree gate pass before commit/push and parent-obligation audit.

## Scope

No new custom borrow checker, target layout copied from Rust, goto renderer,
or C/Java heap admission. The next stage consumes these checked obligations
through existing typed target AST and certification boundaries.
