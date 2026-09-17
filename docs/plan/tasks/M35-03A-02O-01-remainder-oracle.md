# M35-03A-02O-01 — Independent binary64 remainder oracle

- Status: complete
- Parent: [02O](M35-03A-02O-floating-remainder.md)
- Depends on: 02N-04

## Contract

Compute truncating remainder using decoded exact integer/rational operands,
never native floating arithmetic for expected results. Preserve the dividend's
zero sign. Classify nonfinite inputs before rational decoding. The implied
quotient is an unbounded integer, never an already rounded binary64 division.

## Definition of done and tests

Hand-derived goldens cover signed zero, nonfinite inputs, both divisor signs,
subnormals, large exponent gaps and quotients that overflow binary64 division.
Systematically check finite result representability, sign, magnitude and exact
quotient identity. Cross-check a deterministic boundary/random corpus against
the pinned native Rust % operator. Prove the corpus distinguishes Euclidean
remainder, nearest-even quotient, rounded division and zero-sign loss.
Bazel/Clippy, full release/lint gate and fresh independent review pass.
This checkpoint adds no source or target admission.

## Evidence

Candidate dad22f0cc327de0c4a8ffa089358ff1c5002d2b1 passed the focused oracle
and pinned-reference Clippy targets in invocation
5fdce3ed-0281-40e4-8619-7c8547361cab (12.291 seconds).

- 68 independently specified signed/category goldens include fractional
  remainder, maximum-finite modulo three, normal/subnormal boundaries,
  enormous implied quotients, zeros, infinities and NaNs.
- 11,416 pairs combine the prior boundary/random corpus, signed focused cases
  and systematic coverage of every finite normal exponent.
- 10,560 finite cases satisfy exact representability, dividend sign, strict
  remainder magnitude and integral truncating-quotient identity.
- All 11,416 native Rust % observations match the independent oracle.
- Incorrect Euclidean, nearest-even quotient, rounded-division, zero-sign
  and swapped-operand variants differ on 3,261 / 4,412 / 651 / 316 / 10,778
  observations respectively. Faults cannot silently pass the corpus.

The same exact tree passed all 881 Linux Bazel release/lint targets in
invocation 4a9f9897-0a5d-4a0c-8565-424b24ae89fa (42.281 seconds;
11 executed, 870 cached). No production admission or renderer changed.
All 138 older generated bundle hashes remain identical.

Fresh independent Sol Extra High review found no core oracle/specification,
wiring or required-proof defect in that exact tree. It checked all nonfinite
rules, dividend sign, exact unbounded quotient, representability, independently
specified goldens, invariant corpus, fault controls and absence of production
admission. Additional individually pinned fault counterexamples were suggested
only as optional hardening; the current corpus already detects all five.

Documentation-only closure is release-gated again before the scoped commit.
Publishing remains blocked by the prior push-approval decision; no remote CI
claim is made.
