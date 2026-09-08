# M34A-11-02D-03 — Dominating C ranges and counted-loop proof

- Status: planned
- Depends on: M34A-11-02D-02

## Goal

Derive the numeric guards that make actual arithmetic and loop progress safe.

## Definition of done

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
