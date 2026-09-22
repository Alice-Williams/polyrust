# M35-03A-02U-02 — C finite-constant foundation

- Status: planned
- Parent: [02U](M35-03A-02U-finite-f64-constants.md)
- Depends on: [oracle](M35-03A-02U-01-constant-oracle.md)
- Specification: [C17](../../specification/typed-generation/languages/c/rust-finite-f64-constants.md)

## Contract

Extend the certified shared constant profile and dependency inventory to exact
const F64 storage with finite F64 literal initialization. Retain original
producer authority, declaration identity and existing platform/resource proof.
Do not admit source constants in this checkpoint.

## Definition of done and tests

Public certificate tests reject mismatched scalar types, nonliteral initializers,
wrong linkage/owners and forged/lookalike imports. Real rendered producer,
forwarder and external-consumer files pass strict GCC14/Zig O0/O2, standalone
header and UBSan checks, matching the independent bit oracle. Compiling
zero-sign, f32-rounding and wrong-value controls fail the oracle. Resource bounds
and old integer/Boolean constants remain covered. Full gate/review pass,
old output/WIP match, then commit/push this foundation separately.
