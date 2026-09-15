# M35-02B-03K-02 — Borrowed clone and independent cleanup correspondence

- Status: complete
- Parent: [M35-02B-03K](M35-02B-03K-owned-clone.md)
- Depends on: M35-02B-03K-01

## Contract

Certify one scalar parameter, one standard Box construction and one standard
clone borrowing that owner. Admit only a specified immutable, straight-line
root-scope grammar, with whole-owner moves and a final scalar dereference/tail
or explicit return. Refine that closed grammar from pinned observations before
implementation; do not silently broaden historical readers.

## Definition of done and tests

- Authenticate original producer, shared-borrow place, concrete clone call,
  fresh destination, both owner chains and ordered drops against the complete
  PostCleanup normal trace; source identity cannot be inferred from counts.
- Expose source bindings/scopes, canonical exit, clone operation and full MIR
  locations to ordinary typed lowering consumers through private certificates.
- Read-original and read-clone variants prove the source survives cloning.
- Wrong borrow kind/place, alias destination, missing/extra writes or calls,
  moved-source reuse and missing/duplicate/swapped cleanup fail mutation oracles.
- Reject extra clones, arbitrary borrows, branches, custom Drop/allocators and
  other payloads until separately specified. Keep abort-mode compilation.
- Query-only/private-field/erasure controls, fresh broad review and the full
  isolated gate pass before commit/push. C allocation failure and native cleanup
  equivalence remain separate M35-02C/D obligations.

## Closed grammar to implement after K-01

- One nongeneric safe Rust free function takes i32 and returns i32. Its canonical
  unlabeled root block has 2..128 immutable let declarations, no let-else, then
  one scalar dereference as a tail or explicit return.
- The first declaration constructs the standard Box from that scalar parameter.
  Exactly one later declaration uses a K-01 authenticated clone of the current
  original-owner binding. Other declarations only move either live owner into
  a fresh source binding; no second clone/allocation or extra scalar computation.
- The final dereference reads either live owner. Both remaining owners must be
  cleaned up in reverse order of their final root-scope binding declarations.
  Shadowing is distinguished by canonical binding identity, never spelling.
- Reuse canonical scope/exit certification and the complete bounded PostCleanup
  trace. Reuse scalar read/type helpers where their exact contracts apply; do not
  force two simultaneous owners into the existing single-owner call state.
- Keep method shared-borrow staging and explicit-borrow/reborrow staging as
  distinct typed evidence variants. Authenticate the complete source place and
  reference types, unique staging assignments, call argument and chronological
  source-operation order. The clone call retains its source trait FnDef and
  arguments; concrete resolution remains the K-01 authenticated Instance.
- Account for all assignments, Box locals, both call terminators, both drops and
  the full normal Return location. No unexamined extra reference uses, replacement
  owner allocations or harmless-looking scalar rewrites are implicitly admitted.

## Test preparation inventory

Positive fixtures include both clone syntaxes, both final read owners, explicit
returns, moves before/after cloning, interleaved owner moves, reversed final drop
order and same-spelling source bindings. Negative source fixtures include extra
allocation/clone/calls, field/temporary/preborrowed receivers, mutation, nested
scopes/branches, scalar computation and unsupported signatures. Mutation suites
must substitute coherent typed owner places as well as individual MIR fields:
same-type wrong-owner borrows, wrong borrow kind, missing/extra/reordered staging,
altered trait arguments, reused clone destination, altered move provenance,
wrong read owner, both drop substitutions and changed normal Return locations.

## Pinned representation adjustment

The first runtime gate `09f04efc-d921-489c-bd35-ae7ea0eeec08` rejected the
positive interleaved-moves fixture with Assignment. Diagnostic gate
`a284716f-79a6-4280-a6c4-ef5798cba702` showed four constant Boolean stores
remaining in PostCleanup despite having no readers. Reuse the existing complete
Boolean definition/use proof from `multiple/residual.rs`; do not ignore arbitrary
assignments or enable unit residuals. Add corruptions that read/borrow the flag,
change its type or replace its constant definition. This does not widen source
grammar or change historical residual rules.

## Implementation evidence

- The canonical root-frame reader moved unchanged from `calls/frame.rs` to the
  common ownership module. Both call graphs and clone bodies use it; historical
  call-graph/runtime format gate `794be3c6-1918-4485-a45f-7e613130a053` passed
  in 15.956 seconds after the refactor.
- Focused `cloning/source.rs`, `calls.rs`, `loans.rs`, `relations.rs` and `mod.rs`
  separate source grammar, exact call/scalar provenance, shared-loan evidence,
  complete two-owner correspondence and the private query-only body interface.
  All new source/test files remain below 500 lines.
- Initial five-target runtime/format/privacy/erasure/arbitrary-MIR gate
  `f186b478-fb06-4fca-a9ac-1d1df06a7e47` passed in 17.977 seconds. The expanded
  fixture now has eleven accepted bodies and twelve rejected source forms.
  Ordinary consumers check canonical exits, bindings/types, both shared-loan
  forms, both final read owners and reverse binding-order cleanup. Eleven clone
  and eleven construction mappings execute.
- Gate `482b500b-8d29-403d-b146-a53ec74822bb` passed runtime/format in 16.674
  seconds. The suite has 501 cloned-MIR/isolated no-reader rejection cases across
  ten selected bodies, plus harmless unread-constant substitution controls.
  Cases include actual same-type wrong-owner reads/borrows, staging changes,
  erased/aliased destinations, missing/duplicate assignments, wrong calls,
  cleanup substitution/order, phase/owner/signature/budget and observed flags.
- Four invalid Rust programs and seven valid-source oracle mutations reject.
  The separate bounded fixture accepts 128 declarations and rejects 129 without
  changing the fixed corruption inventory. No output package is published.
- The exact inventory assertion, full historical isolated/native/lint gate and
  fresh independent broad review remain required before commit/push.

## Independent review repairs

The initial exact tree `b5e2eb371bdefed27bf6042f5af71caae63ebd16` passed
486 tests / 619 targets in isolated gate
`b5b0fac8-8753-4d8f-96e4-45216901cfff` (52.116 seconds). A fresh Sol Extra
High review found no relation admission defect, but identified three required
proof gaps. All were accepted and repaired:

- Ordinary sibling consumers now match projected producer locations to exact
  HIR-derived Call identities/arguments and destinations, subsequent bindings
  to whole-owner Move chains, and the final scalar read to return-place Copy
  through the selected owner's pointer cast. The binding-order invariant is
  documented at its query API.
- Separate corruptions retain the clone DefId while replacing trait arguments,
  substitute another live owner for a Move, reuse a previously moved source,
  and relocate Return ahead of cleanup. The exact count is now 528.
- A field-receiver body is explicitly rejected: eleven positive and thirteen
  negative source bodies are checked.

Focused runtime/format gate `28b6022c-743d-48ae-8cd0-987635dcc19f` passed
in 16.152 seconds. Full isolated verification and a new independent reviewer
remain required for this repaired tree.

## Closure evidence

- Repaired exact tree `c51b61e927235234d3e801019b01f91a5c5692ba` passed
  isolated gate `7a2934a1-6fb9-428f-a5ad-2846ca2e384e`: 486/486 tests,
  619 targets, 54.578 seconds. All 2,311 Git blobs and executable modes were
  verified before extraction; valid cached test results remained enabled.
- A new independent Sol Extra High reviewer found no remaining core defects.
  It checked the closed source/body relation, repaired consumer projections,
  provenance mutations, inherited isolated receiver tests, private API boundary,
  complete cleanup and absence of target heap admission.
- All twenty preserved ownership files matched their baseline. The final
  documentation-inclusive tree and full gate are recorded in the checkpoint
  commit. Native C allocation/cleanup proof remains M35-02C/D, not this step.
