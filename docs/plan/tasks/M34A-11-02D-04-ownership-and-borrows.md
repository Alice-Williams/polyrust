# M34A-11-02D-04 — C ownership, allocator and borrow dataflow

- Status: planned
- Depends on: M34A-11-02D-03

## Goal

Derive storage provenance, initialization and lifetime facts on actual paths.

## Definition of done

- Track Empty/Live/Moved/Dropped handles and Uninitialized/Prefix/complete storage
  separately, with concrete types, allocator identity, extents and active members.
- Authenticate AllocationRestore against the actual successful allocation result,
  extent/alignment and exact allocator; restoring a pointer never creates Live.
- Require initialized extent, nonnull/alignment and range before every pointer
  formation/read/write. Track read-only borrows and reject owner mutation/drop,
  borrow escape, shallow owning copies and inactive union reads.
- Derive custom/default allocator selection and exact nested forwarding. Same
  callbacks with another context, source allocator substitution or nested null
  do not satisfy invocation identity.
- Model partial construction, commit, move, drop and cleanup at actual writes,
  calls and crossed-scope exits. Check every return/error/cleanup path and join;
  fail closed for effects whose generated summaries are not yet established.
- Preserve the public boundary validation ladder, unchanged rejected outputs,
  clone A-from-B provenance and allocation-free drop obligations.

## Tests and proof

- Use-after-move/drop, double ownership/release, self-move, borrowed alias/escape,
  partial and zero initialization, wrong restored type/extent/allocator.
- Writes may initialize fresh storage; reads cannot precede the exact prefix or
  member fact. Active-union and bound-before-pointer controls are independent.
- Combined boundary failures, sentinel outputs, early exits, loop/branch joins
  and null/default/foreign-context nested allocator substitutions.
- Focused typed/mutation/compile-fail tests, full cached tracked/release/lint/
  eight-target gates; allocator fault injection remains additionally required
  against native generated lifecycle bodies at 06.

## Commit gate

Record evidence and commit/push M34A-11-02D-04. Registered ownership labels are
not proof; unsupported effects remain diagnostics, never optimistic transitions.
