# M34A-11-02D-04C-02A — Fixed typed heap objects

- Status: complete
- Depends on: M34A-11-02D-04C-01

## Goal

Bind a live actual default allocation to one fixed object layout and reuse the
authenticated storage paths for its fields, fixed arrays and union members.

## Definition of done

- An action-root restore establishes a private binding only after checking the
  actual allocation's success, original bytes, measured alignment and allocator.
  Nested restores may reuse an established binding but cannot establish one.
- A repeated compatible restore reuses the same root and initialization; a
  different descriptor or void round trip cannot create a fresh object. A
  conflicting type at a control join cannot regain an unbound allocation.
- Only actual initialized subobjects can be read. Pointer indexing respects the
  fixed original subobject extent; spare allocation bytes grant no extra extent.
- Release accepts the default base, including its restored base, and invalidates
  all surviving heap/interior aliases, including aliases inside another object.
- No numeric loss history is discarded, and heap values cannot yet establish
  allocation sizes or dynamic index facts. Such consumers remain fail-closed.

## Tests and proof

- Positive scalar/record/fixed-array/union initialization and base cleanup.
- Null/unguarded restore, too few bytes, wrong nominal or nested pointer type,
  descriptor mismatch, repeated restore and incompatible branch bindings.
- Uninitialized and inactive-member reads, interior free, expired subobject
  pointers, heap-to-heap aliases and separate allocations with identical layouts.
- Private-boundary tests; focused C/Rust/Bazel lint/docs gates; complete cached
  tracked, release and eight-target conformance gates; uncapped independent review.

## Commit gate

Commit/push 02A only with exact evidence. Numeric-memory and dynamic-buffer
composition remain required by 02B/02C; this is not a rendering certificate.

## Completion evidence (2026-09-09)

- Actual default allocation success, original bytes/alignment and descriptor
  admit one fixed private heap binding. Root-action establishment and nested
  reuse never initialize storage or manufacture a second allocation.
- Scalar, record, fixed-array and union paths reuse initialization/extent facts.
  Raw/restored base cleanup expires all aliases, including interior pointers and
  pointers stored in another heap object. Global escapes remain rejected.
- The final audit follows actual graph edges and checks actions before secondary
  resource-exit errors. This preserves leak checks without obscuring an invalid
  restore behind a later leaked allocation diagnostic.
- The first independent Sol Extra High review found a compatible-type branch
  join defect and missing localized typed-loop evidence. Both were accepted.
  The join regression failed before repair in invocation
  4bea2bb5-c1fa-48dc-9cbf-751a4eeb0962 (418/419 units passed).
- The repair authenticates and normalizes pointee storage identities before
  binding: ABI aliases/immediate qualification coalesce consistently in bindings,
  roots and pointer paths. Nominals, bounds and deeper qualifiers remain exact.
  A 156-type / 24,336-pair oracle checks equivalence with the existing compatible
  type relation, with callback alias/origin and idempotence controls.
- Branch regressions prove initialized scalar/array reads across alias/qualifier
  joins and reject a missing branch write. Typed same-site loop fixtures prove
  complete repeated construction/release, expired base/interior aliases, and
  rejection when later activations skip initialization.
- Final focused gate 025db503-406f-4b15-bff8-6153e322d22d passed: 422 C units,
  typed compile-fail, Clippy, documentation and Buildifier.
- Full tracked gate b0949246-b209-4940-9e7e-66abd40b957a passed all 320 tests;
  release gate bb3e6a42-a011-4703-8c96-380da603538b passed all 257 tests.
- Conformance 7482533a-dba4-4fce-a1df-47d652895a5a passed 50 cases plus one
  portable test: evaluator and eight targets agree, with byte-identical repeated
  manifests. Existing C conformance still exercises the explicit legacy emitter.
- A different independent Sol Extra High reviewer directly inspected the entire
  repaired checkpoint and returned PASS with no substantive production defect
  or required proof gap. The sequencing prerequisite rejects a restore wrapped
  around an allocation call; the admitted form materializes the raw result first.
- The production/test tree remained frozen throughout the second review and
  final full gates. Only this closure evidence changed afterward; documentation
  and Buildifier are rerun before commit. Parent 02 and C migration remain open.
