# M34A-11-02D-04C-02 — Typed allocated storage and dynamic extents

- Status: planned
- Depends on: M34A-11-02D-04C-01

## Goal

Compose actual allocation provenance with type/layout, initialized subobjects
and fixed or dynamic element extents.

## Definition of done

- Restore only an actual successful allocation with sufficient checked bytes,
  measured alignment and exact allocator. Retain original allocation identity
  through copies and void conversions; casts never create another allocation.
- Establish one consistent effective object/element type. Wrong nominal type,
  layout, qualifier level or restored subobject cannot acquire fresh storage.
- Reuse authenticated member/index paths and initialization facts for heap
  storage. Reads require initialized selected fields/elements and active union
  members; writes may initialize fresh live storage.
- Dynamic buffers bind the actual allocated count and element-size product.
  Actual dominating comparisons and counted-loop evidence prove indices and
  initialized prefixes; interval minima or claimed counts are insufficient.
- Release invalidates all heap/subobject pointers and dependent observations;
  partial cleanup touches only established initialization.

## Tests and proof

- Wrong extent/alignment/type/allocator, unguarded or null restore, equal-type
  different allocation, copied/expired results and nested subobject controls.
- Useful scalar/record/array construction and reads, dynamic count boundaries,
  missing/skipped/reordered prefix writes, inactive union and stale-count cases.
- Focused/private-boundary plus full cached tracked/release/lint/conformance
  gates and independent uncapped review.

## Commit gate

Commit/push 04C-02 only with exact evidence; semantic owner transitions remain 03.
