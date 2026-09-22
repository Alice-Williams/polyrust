# M35-03A-02T-01 — Independent signed-widening oracle

- Status: complete
- Parent: [02T](M35-03A-02T-signed-widening.md)
- Depends on: [02S](M35-03A-02S-wrapping-multiplication.md)
- Specification: [shared](../../specification/typed-generation/rust-signed-widening.md)

## Contract

An independent integer model returns the original signed i32 value as an exact
mathematical integer. Deterministic inputs cover both endpoints, zero, neighbours
of signed powers of two, narrow-width boundaries, alternating bits and random
samples. Do not derive expected values from a target renderer or production cast.

## Definition of done and tests

Pinned native Rust agrees using both `as i64` and `i64::from` at O0/checks-on and
O2/checks-off. Pin corpus size and boundary membership. Independently model zero
extension, premature i16 narrowing and disconnected-zero results; require each
to disagree with truth. Bazel owns all reference binaries and tests. Full gate,
Rust/Bazel lint, independent review and preservation checks pass before the
separate commit/push. No production target or source admission in this step.

## Implementation and gate evidence

Tree 6bb56422f8406d41da131710d9214bde298aebe6 passes all five focused targets,
06f9b4db-13c8-4ef1-b0f5-47a5e420b0a5, and all 969 Linux release/lint targets,
8f04fc55-be89-404c-9d90-9f98585448b0 (10 executed, 959 cached). An initial
focused command misspelled the buildifier label; the corrected command includes
the actual root-level lint target and passes. No code or test was disabled.

The reviewed inventory is pinned to 73,890 unique signed inputs: every i16 value,
i32 endpoints, all valid signed-power neighbours, alternating bits and 8,192
deterministic full-width samples before deduplication. Exact unbounded identity
truth agrees with both native `as` and `From` spellings in both build settings.
Zero-extension, premature narrowing and disconnected-zero faults all differ.

All 406 previous generated file hashes and 38 unrelated WIP hashes match; the
isolated checkout matches its scoped index. Independent whole-scope Sol Extra
High review is clean: no actionable finding, core error, required proof gap or
optional feature request. It independently recomputed 65,698 unique non-LCG
inputs plus 8,192 unique LCG inputs with no overlap, and checked both signed
halves, fault models, exact native output and unchanged production admission.

Documentation-only closure receives a final full cached gate before separate
commit/push. C target widening begins only after this checkpoint; the compiler
still rejects casts outside its previously admitted source subset.
