# M35-03A-02Q-01 — Independent wrapping-addition oracle

- Status: complete
- Parent: [02Q](M35-03A-02Q-wrapping-addition.md)
- Specification: [shared](../../specification/typed-generation/rust-wrapping-addition.md)

## Contract

Define independent signed modulo-2^32/2^64 truth and a deterministic pair corpus.
Do not compute expected results with the generated code or native wrapping_add.
Include extrema, same/opposite signs, zero, powers, carry boundaries, asymmetric
pairs and full-width deterministic samples.

## Definition of done and tests

Known identities and boundary vectors pass; native pinned Rust wrapping_add
agrees for every corpus pair at both widths. Oracle controls detect wrong width,
lost carry and safe wrong-operation/saturation results. State clearly that
operand reversal cannot be detected by addition values alone; source trace
proof is mandatory later. Keep support code focused and independently cached.
Full Bazel/lint gate, fresh clean review and a separate tested commit/push.

## Implementation evidence

The independent Python oracle uses unbounded integer addition and signed
modulo normalization. Its 15,790 unique width/operand pairs cover extrema,
cross-sign grids, power/carry boundaries and 4,096 deterministic full-width
pairs per width. Known boundary identities, range/congruence, additive inverse
and commutativity checks accompany four deliberately faulty arithmetic models:
saturation, carryless XOR, subtraction and narrower-width normalization.

The native Rust reference is compiled twice through pinned Bazel toolchains,
with explicit optimization/overflow-check configurations 0/yes and 2/no.
Both actual wrapping_add streams match the independent oracle exactly.
Focused oracle, Clippy, rustfmt, buildifier and docs tests passed under
9953a714-66e7-4767-910e-6611c4716740.

Exact tree 1c87e20e9f9b691a73aba38f45dd0bba96fc10b2 passes all 906
release/lint targets, 83ee9a88-267b-4ab5-ae99-1a2272f02d7a
(10 executed, 896 cached). All 355 older generated package files and all
38 preserved ownership/adjacent WIP hashes remain unchanged. No compiler
admission, target certificate, renderer or vendored upstream file is changed.

Independent Sol Extra High review of the exact tested tree found no actionable
defects in the oracle, corpus, fault controls, native configurations, mathematical
target specifications or Bazel wiring. C/Java foundations and compiler admission
remain separate, unfinished checkpoints; this completion does not imply them.
