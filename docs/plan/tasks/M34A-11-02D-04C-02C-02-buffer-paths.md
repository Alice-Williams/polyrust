# M34A-11-02D-04C-02C-02 — Dynamic storage and guarded paths

- Status: planned
- Depends on: M34A-11-02D-04C-02C-01

## Goal

Restore proved dynamic allocations as element storage and use their original
count for actual guarded access without fabricating fixed C array types.

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
- Private evidence boundary, all cached focused/tracked/release/lint/eight-target
  gates and uncapped independent review; document, commit and push only on pass.
