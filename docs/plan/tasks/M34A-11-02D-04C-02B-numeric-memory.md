# M34A-11-02D-04C-02B — Coupled numeric memory

- Status: planned
- Depends on: M34A-11-02D-04C-02A

## Goal

Compose actual numeric writes and reads with authenticated memory provenance
without laundering arithmetic loss through a heap field or indirect alias.

## Definition of done

- Share existing numeric transfer rules with the storage analysis; do not
  duplicate arithmetic semantics or rewrite the source AST to obtain evidence.
- Retain written numeric domains and arithmetic/call/global loss history across
  scalar and aggregate copies, aliases, joins, lifetime expiry and selected reads.
- Discharge a memory-read obligation only from its actual initialized storage
  and actual reaching writes. Initialization alone cannot erase numeric history.
- Actual later comparisons refine only current observations; mutation and release
  invalidate dependent relations. No optimistic circular proof is admitted.
- Keep standalone numeric diagnostics and actual immutable site authentication.

## Tests and proof

- Useful checked heap size fields and indirect scalar arithmetic/guards.
- Wrapped size stored/read/copied through heap and aggregate paths still rejects;
  stale guards, changed aliases, ambiguous writes and release cannot retain facts.
- Differential transfer-kernel controls, private boundary checks and full cached
  focused/tracked/release/lint/conformance gates plus uncapped independent review.

## Commit gate

Commit/push only with exact evidence. Dynamic extents remain 02C.
