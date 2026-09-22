# M35-03A-02W-04 — Checked Rust character source integration

- Status: planned
- Depends on: [Java foundation](M35-03A-02W-03-java-characters.md)
- Specification: [shared](../../specification/typed-generation/rust-character-values.md)

## Contract

Admit canonical rustc Char literals/types through a private checked input
carrying Rust char. Add an executable character-value mapping and distinct
source TypePlan::Char, preserving source identity even where Java uses Int.
Extend immutable places, scalar signatures, direct calls, conditionals and
same-Char equality/ordering exhaustively. Keep constants, casts and methods
outside this increment.

## Definition of done and tests

Use original multi-crate Rust with public/private APIs, documentation, aliases,
named imports and source-owned target packages. Compare native source and C/
Java across the complete scalar corpus, literal boundaries and comparison
pairs; measure original call order and once-only evaluation. Prove exact typed
U32/Int mappings, original declaration/type/owner joins and manifests that
distinguish char from integer and document the valid-scalar foreign-input
domain. Preserve scalar-field record behavior or diagnose unsupported shapes;
do not silently reinterpret source integers as chars. Atomic negatives cover
casts, constants, methods, references, invalid Rust literals and unsupported
forms. Detect actual value/order/type/owner faults. Export real examples,
update partial inventory, preserve old output/WIP, pass full Linux release/lint
and fresh broad review before separate commit/push.
