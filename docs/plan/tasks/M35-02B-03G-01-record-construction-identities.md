# M35-02B-03G-01 — Authenticate owned record construction

- Status: complete
- Parent: [M35-02B-03G](M35-02B-03G-partial-owned-records.md)
- Depends on: M35-02B-03F
- Specification: [record identity stage](../../specification/typed-generation/languages/c/rust-owned-record-fields.md#record-identity-stage)

## Contract

Add a compiler-session record-construction capability input for canonical,
complete named-field local structs containing only the standard Box<i32> type.
Retain nominal record identity, instantiated type, field declaration IDs/indices,
declared types and initializer expressions in source order. Type aliases and
renamed imports may identify the same nominal record; spelling is not authority.

## Definition of done and tests

- Private safe admission requires the canonical HIR node, correct body owner,
  complete nonempty bounded field coverage and matching compiler-resolved
  field types. Reject generic, external, tuple/unit/enum/union, custom Drop,
  custom representation, update syntax and nonstandard Box payload/allocator.
- Add an executable OwnedRecordConstruction mapping slot to the consuming
  owned-program builder. Registration can occur before/after the Box slot;
  Box-only users retain their existing build and missing/duplicate guarantees.
  Building owned-program bindings still requires the base Box construction
  binding; absent record binding cannot claim record support.
- Positive fixtures check declaration versus source initializer order, aliases,
  renamed imports and same-spelled records in different modules. Negative
  shapes, canonical-node/owner substitutions and missing/duplicate fields fail
  at the stated source/compiler boundary.
- Runtime invokes both typed bindings; compile-negative tests cover missing,
  duplicate/wrong capability/input/context/output and private input/field
  construction. Require exact diagnostics and a nonempty fixture inventory.
- Preserve all prior ownership and native C/Java/lint gates, run fresh review
  and isolated exact-tree full gate, document evidence, commit and push.

This input authenticates one source operation, not field producer provenance,
MIR aggregate correspondence or cleanup. Those remain required G parent work.

## Current evidence

- Historical Box constructor gate `0395b3f6-c745-47c2-b383-cecee92e2e68`
  passed 9/9 targets after the typed optional record slot was introduced.
- Record gate `2a31aa9c-2ca2-4fb3-89fa-ff08fea364bb` passed 11/11 tests
  in 17.955 seconds, including nine exact compile-negative contracts, runtime
  and formatting. Compiler-adapter Clippy continues to deny warnings.
- Nine positive source fixtures and sixteen exclusions/non-record controls retain actual field
  indices/DefIds/normalized types and initializer pointers, including 1/128/129
  field bounds. Aliases share nominal identity; same-spelled module records do
  not. Both registration orders invoke the stored mapping; missing record/base,
  duplicate or wrong typed bindings cannot compile.
- Copied HIR nodes, wrong owners and non-record expressions reject. Missing and
  duplicate fields fail Rust analysis with E0063/E0062; moved-twice values fail
  with E0382 before proof output. Changed initializer order/representation and
  empty inventories trigger the independent fixture oracle.
- Initial isolated tree `e45c32c05b3f102539e63662c9cdb9578ad42353` passed
  all 426 tests across 550 targets in gate `40a400a6-26f1-4bf6-b4fb-1de1053e8ead`.
- Fresh review found one valid test gap: unit structs, unions and nonstandard
  allocators were rejected but lacked explicit negative evidence. Added stable
  unit/union fixtures and a compiler-type allocator oracle. The oracle keeps
  the actual Box declaration and i32 payload fixed, changes only its allocator
  from Global to the fixture's compiler-resolved System type, and exercises the
  production field predicate with matching field/initializer types. This is
  not a claim that stable Rust accepts allocator-api source programs.
- Follow-up gate `a6e747da-b183-4664-8e28-3ed5e1defd9e` passed all 11 selected
  runtime/format and historical constructor-contract tests in 17.381 seconds.
  No aggregate ownership, partial cleanup or target heap output is admitted.
- Corrected isolated tree `47053723a400b0c90ab558bd297dae918d9b4c8d` passed
  all 426 tests across 550 targets in gate `5e2bd759-1856-489b-8dce-1fb579fe9fe8`
  (41.043 seconds), including prior ownership, C/Java native and lint gates.
- A fresh Sol Extra High read-only re-review found no actionable core defects.
  Its optional suggestion to isolate the external-record predicate with a
  second-crate nongeneric fixture is deferred: the existing compiler-resolved
  external fixture is rejected, canonical locality is checked directly, and
  independent isolation of every conjunct is not this task's exit criterion.
  This does not weaken the external-record exclusion.
- The exact closure tree and final full-gate invocation are recorded in the
  checkpoint commit message. Unrelated M34 work and generated/cache outputs
  remain outside that tree. G parent aggregate/cleanup work remains open.
