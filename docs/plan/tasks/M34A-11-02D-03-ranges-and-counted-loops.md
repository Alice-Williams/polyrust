# M34A-11-02D-03 — Dominating C ranges and counted-loop proof

- Status: in-progress
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
