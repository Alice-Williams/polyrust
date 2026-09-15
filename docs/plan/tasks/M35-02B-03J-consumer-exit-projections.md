# M35-02B-03J — Audit structured exit projections before target mapping

- Status: complete
- Parent: [M35-02B-03](M35-02B-03-structured-owned-places.md)
- Depends on: M35-02B-03I-02
- Specification: [owned exit projections](../../specification/typed-generation/languages/c/rust-owned-exits.md)

## Contract

Audit every completed ownership certificate as an ordinary lowering consumer.
The consumer must distinguish canonical source tails and explicit returns, know
the enclosing scope and use the authenticated normal return location. Private
retention of source exits is insufficient if the consumer cannot inspect them.
Do not recover source structure from MIR block edges or invent source bindings.

This is evidence-interface hardening, not a new source shape or target heap path.
Keep all existing source/MIR admission checks and privacy boundaries intact.

## Initial audit

| Certificate | Current exit/return visibility | Required work |
| --- | --- | --- |
| LinearOwnedBody | Tail-only grammar; no public exit/normal-return projection | Retain and expose the canonical tail and matched return |
| MultipleOwnedBody | ScopeFacts retains exit privately; matched return discarded | Expose exit and retain the already-checked return |
| ReturningOwnedBody | ReturnEvidence exposes expression, value, scope and return | Recheck consumer assertions; preserve API |
| Guarded paths | Per-path exit and return exposed; fixture checks reconstruct exits without comparing the projection | Add direct canonical exit comparisons for both arms |
| Early/selection paths | Per-path source exit and normal return exposed and checked | Re-run both outcomes through ordinary consumer tests |
| RecordOwnedBody | Both tail/return forms retained privately; return discarded | Expose SourceExit and checked return |
| BoxedRecordBody | SourceExit and return exposed after H-02 review | Keep existing assertions |
| OwnedCallBody | Both exits and return exposed in I-02 | Keep entry and leaf consumer assertions |

## Definition of done and tests

- Document the complete certificate/exit mapping after inspecting each concrete
  API. No source-shape support claim changes as a side effect of the audit.
- Every admitted form has an ordinary external-to-proof-module consumer test
  comparing its exposed canonical HIR exit with actual source structure and its
  MIR location with the checked Return terminator. Cover nested tail scopes,
  explicit returns and both outcomes of each admitted branch family.
- Retain already-authenticated compiler locations at construction rather than
  querying alternate MIR phases or reparsing source. Source owner/scope mismatches
  and normal-return substitutions remain covered by negative proof controls.
- New evidence fields stay private; historical compile-negative contracts and
  operation mappings continue to pass. No safe arbitrary-MIR constructor appears.
- Fresh broad independent review, focused targets and a full isolated exact-tree
  historical/native/lint gate precede the dedicated commit/push.

Before closing M35-02B, audit its remaining obligations separately, including
clone identity/correspondence required by M35-02C/D. This task does not waive those
obligations or close the parent based solely on interface completeness.

## Planned implementation boundaries

1. Preserve the already-certified Exit in ScopeEvidence, rather than duplicating
   the canonical HIR walk in LinearOwnedBody. Its public body projection can then
   use the same SourceExit enum as the other proof families.
2. Carry the existing trace return location through the linear and record
   relations' Matched structs, and through MultipleOwnedBody::from_matched.
   Do not change the control-flow checker or requery a different MIR phase.
3. Add read-only exit/return getters to the three incomplete body APIs. Existing
   Returning/Guarded/Early/Selection wrappers keep their typed interfaces.
4. Extend the dedicated linear, scope, multiple, record and guarded fixture
   assertions. Keep these tests in their existing Bazel targets so each proof
   family retains its normal independent test result.
5. Run those focused targets plus early/selection/boxed-record/call-graph consumers,
   then all compile-negative and full release/native/lint gates on the exact tree.

## Implementation evidence

- ScopeEvidence now retains the canonical Exit it already certified. The shared
  SourceExit enum is re-exported at the ownership module boundary; no duplicate
  syntax type or additional source parsing is introduced.
- Linear and record relations carry their existing checked return location.
  MultipleOwnedBody retains its matched return, and guarded paths consume that
  retained field when splitting body evidence. Read-only body getters expose
  exits/returns without adding construction or mutation APIs.
- `test/owned_exit_consumer.rs` checks canonical compiler expression pointers,
  exact source owner/scope, tail versus explicit return, and the complete normal
  MIR Return location. All ten proof families exercise it through their existing
  independent Bazel targets, including both branches and nested scopes.
- The shared helper's compiler-session lifetime is explicit. The call-graph
  consumer was adjusted to preserve that same lifetime rather than use unrelated
  anonymous lifetime parameters.
- Focused gate `b47b0459-7c9c-4b6b-9566-4851e8941462` passed all eleven
  targets in 23.715 seconds: ten runtime proof families and the common formatting
  boundary. Their existing source/corruption inventories remain unchanged.
- Full isolated gates and both broad reviews passed as recorded below.

## Regression fixture repair

Initial tree `e9bf6b4476407a061b20441a294fa31210a4638c` passed 466 tests;
three compile-negative targets failed to build in full gate
`7b00ba26-a254-4cd3-a939-e3eb49e05be2`. Their fabricated struct literals
omitted newly retained private fields, changing the compiler diagnostic shape.
The fixtures now supply those fields through read-only getters, so they test
privacy rather than missing-field construction. Exact E0451 expectations remain
unchanged. Repair gate `9f5d62f4-c99f-43ac-a266-4a04c8ce1413` passed all
three privacy contracts and both affected formatting targets in 13.813 seconds.

## Closure evidence

Repaired tree `30d89af44381310d12542de01dd237d59a947faa` passed full isolated
gate `379edd17-7b8d-4fbf-802f-8ae0ed0d5bd9`: 469/469 tests, 599 targets,
23.089 seconds, with all 2,277 Git blobs and executable modes verified.
Independent Sol Extra High reviewer `owned_exit_projection_review` confirmed
the fixture repair and found no other core defect. Fresh Sol Extra High reviewer
`owned_exit_projection_recheck` independently found no core correctness,
contract, privacy-boundary or coverage issues in that immutable tree.

The review checked all ten certificate families, canonical exit/scope identity,
both branch outcomes, complete Return locations and the unchanged query-only
construction boundary. This closes the evidence-interface audit only. Clone
and C/Java heap work remain open. Closure documentation receives a final isolated
full gate before commit; the commit records that exact tree and invocation.
