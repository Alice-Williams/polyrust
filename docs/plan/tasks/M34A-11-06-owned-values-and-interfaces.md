# M34A-11-06 — Implement owned aggregates and flat C interfaces

- Status: planned
- Depends on: M34A-11-05

## Goal

Cover the complete portable value algebra and polymorphism with verified ownership.

## Definition of done

- Monomorphize immutable list, option/result, record, payload-free enum and legacy payload-enum compatibility representations by typed identity.
- Implement exact construction, observation, recursive semantic equality, separate portable-expectation comparison, clone/move/drop and initialized-prefix cleanup.
- Implement the entire owning Interfaces bundle, private flat vtables, exact implementation witnesses and explicit field delegation.
- Allow zero-implementation interfaces without fabricated values or foreign vtable registration; support every nested admitted type position.
- Preserve value semantics across independent conformances and allocations; expose no mutable public backing layout.
- Implement c/runtime-traversal.md: iterative clone/construction/comparison,
  allocation-free intrusive destruction/rollback, exact work tags/ownership,
  checked node counts and no recursive interface callback re-entry.
- Use the exact private clone/drop work-parameter lifecycle slots; authenticate
  one-step enqueue semantics, persistent destinations and outer-driver commit.

## Tests and proof

- Add c_ownership_fault_test and c_public_abi_test against the exact
  c/callable-abi.md and c/public-value-abi.md contracts, with allocation-site inventory and both optimizer/
  sanitizer matrices from c/platform-and-proof.md. Include interface values
  in legacy payload-enum fields and zero-implementation interface nesting.

- Native separate consumers for aggregate construction/projection/equality, each tag, nested ownership and independent clones.
- Distinct-NaN-payload and signed-zero matrices through every aggregate;
  representation-preservation audits separately require exact raw bits.
- Multiple/empty interfaces, overlapping method names, exact concrete/dynamic dispatch, interface returns and nested lists/options/results.
- Recursive I / R(child: Option<I>) with R implementing I: R(None) and finite
  nested Some values, clone/drop/dispatch, every allocation-failure prefix,
  ASan/leak/UBSan and finite deterministic specialization inventories. Legal
  callable cycles must not be rejected or expanded indefinitely.
- Adversarial wrong-table/signature/receiver/tag/cast mutations; allocation failure at every prefix/clone/dispatch step.
- Swapped/missing work-engine types, different engine identity, expired queued
  destination slots and premature callback-to-public commit are rejected.
- Every nested direct/interface/helper/work allocation is charged to the one
  effective invocation allocator. Fault controls detect default substitution,
  changed callback context and allocator A clone of allocator B source storage.
- Empty-source/double/self move, null slot addresses and nonempty destinations
  use the exact status precedence and unchanged-slot controls in callable-abi.md.
- Pairwise combined detectable failures follow the global ABI validation
  ladder; preserve sentinel outputs and prove no later callbacks/reads occur.
- ASan/leak/UBSan, deterministic package layout and full tracked/release gates.
- Deep API-created recursive record/interface values, small-stack native runs,
  equality/list-search/NaN-class expectation checks, temporary work allocation
  faults, count boundary/one-over mutations and allocation-disabled drop.

Test targets required by this slice must be added before it closes; proposed
future targets are not evidence of an existing implementation. All builds and
tests run in the Linux development container with pinned toolchains and normal
Bazel action/test caching.

## Commit gate

Record exact commands, counts, failures and dispositions here. Commit and push
this completed checkpoint with M34A-11-06 in the message. Keep the overall
C migration open until M34A-11-09; never substitute a partial slice for Pass.
