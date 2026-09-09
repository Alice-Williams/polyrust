# M34A-11-02D-03 — Dominating C ranges and counted-loop proof

- Status: complete
- Depends on: M34A-11-02D-02

## Goal

Derive the numeric guards that make actual arithmetic and loop progress safe.

## Definition of done

02D-02 closes at 70e52f5923bbc8356a22537d3774989f37a93775 after independent
clean review and all cached gates. The detailed contract is
[numeric range proof](../../specification/typed-generation/languages/c/numeric-range-proof.md).
Implement these cohesive checkpoints in order:

1. [03A: numeric domains and transfers](M34A-11-02D-03A-numeric-domains.md)
2. [03B: counted-loop evidence](M34A-11-02D-03B-counted-loop-evidence.md)
3. [03C: numeric flow and composition](M34A-11-02D-03C-numeric-flow.md)

This breakdown keeps interval arithmetic, loop path proof and fixed-point flow
independently reviewable. It does not defer any parent requirement beyond 03;
the parent closes only with all child checkpoints and the checklist below.


- Derive typed scalar ranges and relational facts from actual edge predicates,
  checked operations and known-call results. Kill facts on writes/escape/effects;
  join only facts valid on every reachable predecessor and converge conservatively.
- Check signed arithmetic/conversions, integer division/remainder/shifts and
  allocation-count products before their operation executes. Do not use host
  overflow, a post-operation guard or an unsigned type alone as size safety proof.
- Prove every clause of c/counted-loops.md from actual initialization, condition,
  writes, aliases and graph paths. Count exactly one authenticated step on each
  continuing path, including Continue; early exits need no step.
- Preserve the zero-iteration exit and distinguish zero/one/multiple iterations.
  Bound is the exact immutable snapshot, counter has no foreign mutation.
- Retain verifier-derived extent/index obligations for the ownership stage;
  scalar range alone does not authorize null or unrelated pointer arithmetic.

## Tests and proof

- Full counted-loop mutation inventory from the normative spec: wrong initial
  value, direction, bound, missing/double/wrong update, bypassing Continue,
  nested substitution and address escape; branch-local single-step positives.
- Direct SIZE_MAX-1/SIZE_MAX arithmetic controls without huge native loops.
- Reversed/non-dominating guards, stale facts after assignment, branch/loop joins,
  signed extrema, checked size products and unknown-condition controls.
- Focused analysis tests plus full cached tracked/release/lint/eight-target gates.

## Commit gate

Record evidence and commit/push M34A-11-02D-03. No generic iteration/arity cap
is added; native finite-loop renderer proof remains stage 04.

## Combined closure evidence

- 03A closed at ee2b571 with checked scalar domains/transfers, exact-value
  independent oracles, boundary controls and clean independent review.
- 03B closed at 6c74c36 with actual counter/bound/write/alias inventory and
  finite path proof of exactly one step per continuing path, including branches,
  nested cycles and Continue. Zero/one/SIZE_MAX controls do not unroll huge loops.
- 03C closes in this checkpoint with actual edge refinement, conservative
  joins/widening, strict converged operation checking, invalidation and retained
  arithmetic/storage origins, algebraic size relations, actual-site obligations
  and the numeric consumer of retained pre/post-step phases.

The combined C unit suite has 317 passing tests. Final focused gate
39476103-f140-41ae-be06-df14a0672da1 includes Rustdoc, Clippy, Buildifier and docs.
Full tracked gate 9e57e76f-6c8e-44f7-8592-b86e93e025d7 passes 320 targets;
release 67372068-dafa-4bd7-9fd2-696d18133499 passes 257; conformance
a01c5ca6-66ba-4de9-9b7a-1b314db75544 proves 50 cases plus one portable test
across eight targets with deterministic manifests. 03C records the accepted
review findings, red regressions, repairs and final clean independent review.

All parent numeric/counting requirements are covered. Private index/extent and
call obligations are retained for 04/05; none is a pointer, initialization,
allocation, lifetime, generated-call or render-ready certificate. Existing
legacy C conformance is regression evidence, not typed-C cutover evidence.
