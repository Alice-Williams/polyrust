# M34A-11-02D-04C-02C — Dynamic extents and initialized prefixes

- Status: planned
- Depends on: M34A-11-02D-04C-02B

## Goal

Prove runtime-sized buffers using actual allocation count relations and
construction progress, not invented fixed C array types or interval minima.

## Definition of done

- Retain original count-times-element-size identity, nonwrapping bytes, concrete
  element layout and allocation origin in a private storage extent.
- Derive index bounds from actual dominating relations or counted-loop phases.
  Later count mutation cannot silently change the existing allocation's extent.
- Track must-initialized elements/prefixes across ordered loop writes and joins;
  skipped or reordered construction cannot claim complete storage.
- Preserve member/interior pointer provenance and invalidate all observations
  on release or conflicting writes. Keep storage shape separate from C syntax.

## Tests and proof

- Runtime counts, zero/maximum/overflow boundaries, nested element layouts,
  guarded random access, counted construction and initialized-prefix reads.
- Off-by-one, stale/replaced counts, missing/skipped/reordered writes, partial
  construction and expired alias controls.
- Full cached focused/tracked/release/lint/conformance gates, private evidence
  controls and uncapped independent review; close parent 02 only after success.

## Commit gate

Commit/push with exact evidence. Semantic owner transitions remain 04C-03.
