# M34A-11-07 — Register all executable C capability mappings

- Status: planned
- Depends on: M34A-11-06

## Goal

Make the consuming builder and exact mapping certificates authoritative for all 42 capabilities.

## Definition of done

- Add one capability mapping file per shared catalogue row with capability-owned typed inputs/plans/outputs.
- Store mandatory checked handlers in CPluginBuilder; exact preflight uses the same slots and prerequisites.
- Map complete declarations/types/values/operations/control/tests and interface bindings; do not route through Java or legacy C.
- Derive range/ownership/cleanup requirements from selected C plans and authenticate their actual output roots.
- Infer local origins, fallible prerequisites, short-circuit sequencing and enums consistently on dynamic and typed paths.

## Tests and proof

- Add c_capability_plan_test and compare all 42 rows and every closed input
  variant in c/capability-inventory.md against the executable registry. Missing
  invocation, mutation or native evidence prevents registering that row.

- Compile-fail missing/duplicate/wrong-capability/wrong-dialect/erased-output/forged-plan contracts.
- Every closed input variant has an invocation and independently compiled/executed operation fixture.
- Mutation of selected strategy, operation, signature, ownership/effect or helper evidence fails certification.
- Remove one slot and prove both typed Supports admission and dynamic preflight reject it; full cached integration gates.

Test targets required by this slice must be added before it closes; proposed
future targets are not evidence of an existing implementation. All builds and
tests run in the Linux development container with pinned toolchains and normal
Bazel action/test caching.

## Commit gate

Record exact commands, counts, failures and dispositions here. Commit and push
this completed checkpoint with M34A-11-07 in the message. Keep the overall
C migration open until M34A-11-09; never substitute a partial slice for Pass.
