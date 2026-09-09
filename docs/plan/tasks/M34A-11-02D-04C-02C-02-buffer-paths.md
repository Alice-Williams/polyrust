# M34A-11-02D-04C-02C-02 — Dynamic storage and guarded paths

- Status: complete
- Depends on: M34A-11-02D-04C-02C-01

## Goal

Restore proved dynamic allocations as element storage and use their original
count for actual guarded access without fabricating fixed C array types.

## Implementation order

1. Introduce a closed private allocation-storage shape and explicit dynamic
   element selector. A buffer container is not a C object place; the restored
   base selects element zero. Keep fixed-object and nested fixed-array paths.
2. Bind only the actual matched count/element layout. Keep sparse element
   initialization separate from C aggregate cells and reuse ordinary nested
   struct/union/array reads, writes and joins.
3. Derive current index/count ordering from actual comparison edges and checked
   counted-loop phases. Keep those relations in the existing numeric state,
   including its mutation, scope, join and lifetime invalidation rules.
4. Track a runtime selected element by its typed current index observation and
   conservative numeric range. A write initializes that selected element, not
   every element in the range. Invalidate dependent observations and pointer
   equivalence when their source storage changes; never retarget a saved pointer.
5. Keep original count snapshots when a scope exits, but retire that count
   activation's relational authority. A new activation of the same declaration
   cannot redefine an old allocation. Define base/zero-offset release explicitly.
6. Run constructor/private-path controls and actual-AST positive/negative tests,
   all cached gates, and independent review before documenting/committing/pushing.

## Definition of done

- Distinct internal dynamic shape and element paths retain count, layout,
  original root, nested subobjects and explicit base/interior release identity.
- Actual comparisons or counted-loop phases relate current indices to the
  captured immutable count. Alias mutation invalidates dependent observations.
- Exact writes/reads reuse initialization, union activity and numeric history;
  ambiguous operations stay conservative. Release expires all derived aliases.
- Prefix/full-buffer construction claims remain rejected until checkpoint 03.

## Tests and proof

- Useful runtime-count guarded reads/writes and nested element layouts.
- Zero/max/overflow, off-by-one, replaced count/index, wrong shape, null restore,
  ambiguous update, uninitialized read, interior free and expired-alias controls.
- Index-source mutation through aliases, changed/scope-reactivated counts,
  saved interior pointers, overlapping symbolic writes, nested unions/arrays,
  branch joins and the still-rejected whole-buffer/prefix construction claim.
- Private evidence boundary, all cached focused/tracked/release/lint/eight-target
  gates and uncapped independent review; document, commit and push only on pass.

## Implementation checkpoint

Private storage shapes distinguish fixed objects from dynamic element sequences;
no runtime count becomes a fabricated fixed C array. Matched allocation requests
retain original count/layout/byte provenance, while typed base restoration,
guarded element paths and nested aggregate/array cells use the same live root.
Base/element-zero release is explicit; arbitrary interior release is rejected.

Current scalar index identities survive range refinement and control-flow joins.
Actual strict comparisons establish current index/count ordering. Mutation,
scope exit, count redeclaration and allocation release retire dependent facts.
Sparse writes establish only the selected element, with conservative overlapping
writes, union activity, saved-address and numeric-loss handling.

Shared observation joins preserve numeric facts, pointer targets, copy rebasing
and subobject overlap identity. Different singleton bindings cannot become
dependency-free constants; physical coverage counts each offset once. Repeated
scope/allocation fixtures establish fresh activation state without reviving old
count or pointer authority.

## Completion evidence (2026-09-09)

- Focused invocation 0402f0a5-16df-4466-805d-b1a9df3e58cb passed 492 C
  units (35 added in this checkpoint), typed compile-fail, Clippy, documentation
  and Buildifier. Two new privacy doctests hide dynamic shape/selection internals.
- Tracked invocation 0c5e14de-5408-44f1-b257-8743a6ad34d3 passed 445 rules /
  320 tests. Release invocation 797a1e08-864d-435b-ab6b-63b247e8e1f1 passed
  all 257 tests.
- Conformance invocation 76d800cc-6d74-489e-8788-aac9585d06aa passed 50
  cases plus one portable test: evaluator and eight targets agree; repeated
  generation produces byte-identical manifests. C still uses legacy emission.
- Regression coverage includes constant/runtime indices; actual/reversed/false
  strict guards; counted pre/post-step access; incomplete/must initialization;
  nested arrays, structs and unions; clean/wrapped numeric history; overlap;
  copied/saved pointers; release and scope/activation retirement; and actual
  ABI-equivalent U64/typedef index bindings.
- The first independent Sol Extra High review found R1: differently refined
  observations of one unchanged binding joined initialization but lost numeric
  and saved-pointer identity. Accepted. Three regressions failed before repair
  (d338e8aa-524a-4c33-8475-45b523aafa0a). Shared observation handling now
  retains every reaching numeric domain/loss; six repair tests pass.
- A different reviewer found R2: distinct singleton bindings could coalesce into
  a dependency-free constant, violating the explicit retirement contract.
  Accepted as a proof-contract defect, not a demonstrated physical-memory
  failure. Two regressions failed before repair
  (19cd0946-f892-4580-abea-5594295ef214). Constant coalescing now requires
  two dependency-free constants. The interacting range-coverage check also
  deduplicates physical offsets and joins their cells conservatively; three
  regression tests include both missing-sibling and complete-coverage cases.
- A third independent Sol Extra High review directly inspected all scoped source,
  tests, specifications, prerequisites and Bazel registrations. Its uncapped
  verdict was PASS with no substantiated current-stage defect or required proof
  gap. It rechecked both repairs and the full count/layout, guarded access,
  initialization, numeric loss, alias, join, activation and lifetime matrix.
- No finding was dismissed or left unfixed. The production/test/spec tree stayed
  frozen through final review and all full gates. Only closure documents changed
  afterward; documentation and Buildifier are rerun before commit.
- Invalid post-step loop access remains rejected. Provisional fixed-point
  poisoning can localize strict replay at an earlier uninitialized loop point;
  the regression records that diagnostic without accepting the invalid program.
- Initialized prefixes remain checkpoint 03. Owner/call contracts and rendering
  remain later stages; parent 02C/02 and C render readiness stay open.
