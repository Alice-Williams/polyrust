# M35-03A-02Q-02 — C wrapping-addition safety foundation

- Status: planned
- Parent: [02Q](M35-03A-02Q-wrapping-addition.md)
- Depends on: [oracle](M35-03A-02Q-01-addition-oracle.md)
- Specification: [C17](../../specification/typed-generation/languages/c/rust-wrapping-addition.md)

## Contract

Certify exact-width unsigned addition and guarded signed normalization using
normal typed nodes, without widening source admission. Preserve existing numeric
obligations: no signed overflow, out-of-range signed casts or misuse of wrapping
values as allocation/extent proofs.

## Definition of done and tests

Both widths certify valid constructions; unsafe missing/reversed/wrong guards,
wrong values, signed-add and unguarded-cast mutants reject. Typed shape checks
catch missing I32 normalization and safe semantic substitutions. Separate
GCC14/Zig O0/O2 plus GCC UBSan consumers match independent modular truth.
Exact imports, original callable authority, ABI headers and resource bounds hold;
no runtime/helper/math-library artifact appears. Previous range/ownership gates,
full isolated release/lint gate, fresh clean review and separate commit/push.
