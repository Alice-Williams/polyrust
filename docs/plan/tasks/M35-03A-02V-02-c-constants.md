# M35-03A-02V-02 — C signed-infinity constant foundation

- Status: planned
- Parent: [02V](M35-03A-02V-infinite-f64-constants.md)
- Depends on: [oracle](M35-03A-02V-01-constant-oracle.md)
- Specification: [C17](../../specification/typed-generation/languages/c/rust-infinite-f64-constants.md)

## Contract

Register the exact typed HUGE_VAL standard constant and signed-negation form.
Separate constant inventory values from finite literal nodes. Extend only
matching F64 constant certification/import facts; keep renderer structural.

## Definition of done and tests

Native owned/imported/aliased objects and generated readers agree on both exact
infinity signs with GCC14/Zig O0/O2 and UBSan. Standalone headers, inferred
math.h and no spurious libm dependency are proven. Wrong sign/type/shape/owner,
resource and forged dependency controls reject. Finite output hashes remain
unchanged. Full gate and clean independent review precede commit/push; no
compiler source or Java admission changes in this checkpoint.
