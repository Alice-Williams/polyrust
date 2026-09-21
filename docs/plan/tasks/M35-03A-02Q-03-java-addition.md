# M35-03A-02Q-03 — Java wrapping-addition foundation

- Status: planned
- Parent: [02Q](M35-03A-02Q-wrapping-addition.md)
- Depends on: [C foundation](M35-03A-02Q-02-c-addition.md)
- Specification: [Java21](../../specification/typed-generation/languages/java/rust-wrapping-addition.md)

## Contract

Admit structural Add for exact primitive Int/Long operands and result with
Additive precedence. Keep existing Double arithmetic unchanged; do not admit
other integer operators, boxing, mixed widths or helper calls.

## Definition of done and tests

Typed shape and malformed type/precedence/operator controls pass. Strict
separate Java21 producer/client compilation matches independent signed modular
truth at both widths; wrong-width/saturation/operation controls are detected.
Original dependencies, API/resource/byte accounting and no-runtime inventory
hold. No frontend admission change. Full isolated release/lint gate, fresh clean
review and separate commit/push.
