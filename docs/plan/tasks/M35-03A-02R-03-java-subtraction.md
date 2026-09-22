# M35-03A-02R-03 — Java wrapping-subtraction foundation

- Status: planned
- Parent: [02R](M35-03A-02R-wrapping-subtraction.md)
- Depends on: [C foundation](M35-03A-02R-02-c-subtraction.md)
- Specification: [Java21](../../specification/typed-generation/languages/java/rust-wrapping-subtraction.md)

## Contract

Admit exact primitive Int/Long Subtract with Additive precedence and recursively
checked operands. Reuse normal target AST, authority, certification and renderer.
Keep existing Add/Double behavior unchanged and unrelated integer operators closed.

## Definition of done and tests

Both widths compile in separate strict Java21 producer/client units and agree
with independent modular truth. Detect addition, reversal, saturation, narrowing
and disconnected-operand faults. Mixed widths, boxed/string/Boolean values,
incorrect result/precedence and missing-import/arity faults reject in both operand
positions. Depth, call-height and byte-budget controls remain active. Full gate,
preserved old output/WIP, fresh review and separate commit/push. No source admission.
