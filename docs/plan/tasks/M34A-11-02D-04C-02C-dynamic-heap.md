# M34A-11-02D-04C-02C — Dynamic extents and initialized prefixes

- Status: in-progress
- Depends on: M34A-11-02D-04C-02B

## Goal

Prove runtime-sized buffers using actual allocation count relations and
construction progress, not invented fixed C array types or interval minima.

## Ordered implementation checkpoints

The [dynamic-buffer specification](../../specification/typed-generation/languages/c/dynamic-buffer-proof.md)
defines the admitted count grammar and proof boundary.

1. [01: typed counts and actual allocation products](M34A-11-02D-04C-02C-01-count-products.md).
2. [02: dynamic storage and guarded access](M34A-11-02D-04C-02C-02-buffer-paths.md).
3. [03: initialized prefixes and construction](M34A-11-02D-04C-02C-03-prefixes.md).

Each checkpoint has its own tests, full cached gates, independent review and
commit/push. Count metadata alone cannot admit dynamic access. Parent 02C and
parent 02 remain open until all three checkpoints meet their definitions of done.

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
