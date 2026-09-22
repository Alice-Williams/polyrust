# M35-03A-02W-01 — Independent Unicode scalar oracle

- Status: complete
- Parent: [02W](M35-03A-02W-character-values.md)
- Specification: [shared](../../specification/typed-generation/rust-character-values.md)

## Contract

Build an integer-only reference independent of the target ASTs, Rust character
APIs and host Unicode databases. Check the complete 0..0x10FFFF interval,
including every surrogate, plus deterministic out-of-range u32 values.
Compare actual safe Rust char construction/transport and six comparisons at
O0/checks-on and O2/checks-off. Observe real compiling narrowing, BMP-only,
surrogate-admission, reversed-order and UTF-16-order faults.

## Definition of done and tests

- Exactly 1,112,064 scalar values are accepted; all 2,048 surrogates reject.
- Noncharacters, unassigned/private-use points, NUL and supplementary scalars
  remain values, without replacement or normalization.
- Safe native Rust construction and exact round-trip values match the oracle.
- Six comparison results match numeric scalar ordering for boundary cross
  products, equal values and deterministic additional valid pairs.
- Actual wrong-width values, wrong admission and wrong-order observations
  agree with independent faulty models and are distinguished from truth.
- Invalid oracle inputs reject; malformed native packets cannot be ignored.
- Separate Bazel targets run the native oracle, Rust Clippy and Rustfmt.
- Full release/lint, independent review and preservation pass before commit.
- No production frontend/target admission changes in this checkpoint.

## Implemented evidence

The native oracle observes all 1,112,064 Unicode scalars, all 2,048 surrogate
values, 80 deterministic out-of-range Unicode probes represented as u32 and 4,453 valid
comparison pairs. The seven actual faulty columns cover sixteen/eight-bit
narrowing, BMP-only admission, surrogate acceptance, reversed comparisons,
UTF-16 lexicographic order and byte-truncated comparisons. Both Rust profiles
agree exactly with independent integer truth. Invalid oracle inputs and five
malformed native packet forms reject. No Unicode database or target AST is used.

Clippy requested fixed-size as_chunks::<12> instead of chunks_exact; the native
protocol now exposes its twelve-byte packet size in the Rust type. Both lint/
format targets and the oracle pass. Full tree
9fa102268eba796f7bc7cdb1d6068107817cde1a passes all 1,015 release/lint targets
(12 executed, 1,003 cached), invocation db8b9b4c-6930-4163-ada4-885166a77cf4.
All 501 prior generated output hashes and 38 unrelated WIP hashes are unchanged.
Fresh broad Sol Extra High review of the exact implementation tree is clean,
with no material findings or review-actionable optional extensions. Documentation
closure is gated again before the separate commit/push. Target foundations
and checked source admission remain separate, unfinished checkpoints.
