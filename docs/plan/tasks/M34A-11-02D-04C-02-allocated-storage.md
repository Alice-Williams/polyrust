# M34A-11-02D-04C-02 — Typed allocated storage and dynamic extents

- Status: complete
- Depends on: M34A-11-02D-04C-01

## Goal

Compose actual allocation provenance with type/layout, initialized subobjects
and fixed or dynamic element extents.

## Ordered checkpoints

1. [02A: fixed typed heap objects](M34A-11-02D-04C-02A-fixed-heap.md) — complete.
2. [02B: coupled numeric memory](M34A-11-02D-04C-02B-numeric-memory.md) — complete.
3. [02C: dynamic extents and initialized prefixes](M34A-11-02D-04C-02C-dynamic-heap.md) — complete.

Each checkpoint requires its own full cached gates, uncapped independent review
and commit/push. The parent remained open until all three passed. In particular,
fixed object capacity is not a substitute for dynamic count relations.

## Closure audit and evidence

02A covers exact successful allocation restore, layout/qualifiers, effective
type, shared physical identity and selected subobject initialization. 02B retains
actual numeric memory history, aliases and conservative joins. Completed 02C
adds original runtime count products, guarded elements and actual construction
prefixes. Release/expiry and failure-path controls compose in the same gate;
partial storage cleanup does not certify semantic owning-child rollback.

The final [02C-03 evidence](M34A-11-02D-04C-02C-03-prefixes.md#final-evidence)
passes 515 C units and companion gates, all 320 tracked and 257 release tests,
and deterministic eight-target conformance. Its complete independent review
explicitly re-audited this parent and found no remaining storage obligation.
Owner transitions, live allocation families, incoming ABI and body-derived call
contracts retain their separate tasks. This closure cannot render new C code.

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
