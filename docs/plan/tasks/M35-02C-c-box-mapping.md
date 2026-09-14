# M35-02C — Lower admitted owned Boxes into typed C

- Status: planned
- Parent: [M35-02](M35-02-rustc-owned-values.md)
- Depends on: M35-02B

## Contract

Implement the agreed closed owned shapes through existing C registry, AST,
capability, verification, linking and rendering boundaries. Add typed allocation,
initialization, move and cleanup bindings, with type-derived imports. Preserve
allocator provenance; include a cleanup strategy for every admitted exit.
No raw C body/import escape hatch or parallel renderer is permitted.

Specify normal-return, clone, partial-move, allocation-failure and panic policy
before enabling each shape. Retain the pinned abort configuration; do not turn
allocation abort into a recoverable result, assume custom allocator behavior,
or quietly admit custom Drop/unwind/unsafe/raw Box conversions.

## Definition of done and tests

- Positive typed AST assertions cover allocation identity, initialization,
  pointer use, conditional cleanup and deallocation ownership/order.
- Missing/duplicate cleanup, wrong allocator, moved-owner use, wrong projections
  and wrong-registry inputs are rejected at their stated boundary.
- All admitted outputs certify through RenderReadyPackage<CDialect> and compile
  under strict GCC/Zig; unsupported Rust diagnoses before publication.
- Java continues to reject owned shapes until it has its own explicit mapping.
- Fresh review and full historical/no-heap gates pass; M35-02D supplies final
  native cleanup equivalence before declaring parent support complete.
