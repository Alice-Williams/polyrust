# M35-03A-02X-04 — Checked Rust character constant integration

- Status: planned
- Parent: [02X](M35-03A-02X-character-constants.md)
- Depends on: [Java foundation](M35-03A-02X-03-java-constants.md)
- Specification: [shared](../../specification/typed-generation/rust-character-constants.md)

## Contract

Extend the private compiler constant witness with Char(char), checking original
type/width/domain after rustc evaluation. Reuse existing constant capability
mappings for declarations, reads, imports and aliases. Preserve original Char
facts, exact values and source owners across both target inventories, including
constant-only owners. Never infer source identity from a target integer.

## Definition of done and tests

Original multi-crate Rust and generated C/Java agree for public/private/local/
inherent constants, named imports and aliases. Include mixed same-valued Char
and I32 declarations, original docs/visibility and constant-only producers.
Typed probes corrupt kind/value/owner/declaration independently while preserving
positive controls; Java Char-to-I32 substitution must fail despite identical
target Int storage. Unsupported generic/trait/reference/type-alias forms and invalid
scalar facts reject atomically with absent/existing destinations unchanged.
Replace newly supported old negative cases with still-unsupported cases.

Measure real producer-value cache invalidation, failure against old truth,
success against updated truth and restored output hashes/cached native success.
Recompile Java dependents after value mutations. Export actual source-owned
examples, update partial parity only, preserve old outputs/unrelated WIP and
pass full Linux release/lint plus fresh broad review before commit/push.
