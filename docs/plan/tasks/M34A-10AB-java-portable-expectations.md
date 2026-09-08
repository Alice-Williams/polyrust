# M34A-10AB — Reconcile Java portable expectation comparison

- Status: planned
- Depends on: M34A-11-01R

## Goal

Repair the concrete comparator mismatch discovered during the C design review,
without changing established portable floating-point semantics.

## Definition of done

- Keep IEEE semantic equality, NaN-class portable expectation equality, and
  raw-bit representation equality separate and explicitly named in typed APIs.
- PortableTests uses expectation comparison for both success and error payloads;
  recursively cover lists, records, options, results and legacy payload enums.
- Any NaN payload/sign matches an expected NaN. Non-NaN raw bits, including
  signed zero, remain exact. Literal/FloatAbs representation audits remain
  payload-exact rather than using the looser portable-test comparator.
- Use structural Java AST, closed runtime/member catalogue variants, derived
  dependencies and authenticated plans; preserve the small-module policy.
- Correct Java/shared specification wording and restore compliance only after
  focused/native/full gates and a fresh independent review.

## Tests and proof

- Demonstrate an evaluator-accepted distinct-NaN-payload portable test failing
  Java before repair; run generated and conformance entry points independently.
- After repair, both entry points execute the exact case inventory, including
  nested success/error payloads, and print matching completion counts.
- Negative controls: finite mismatch, NaN versus finite, signed-zero mismatch;
  raw-bit audit still rejects different NaN payloads.
- Rust/mapping/compiler tests, Java 21 lint, native public consumers, resource
  reservations, all tracked Bazel rules, release gate, eight-target conformance,
  hosted CI and fresh Sol Extra High review. Normal caches stay enabled.

## Commit gate

Record exact failing controls, successful invocations, review dispositions and
immutable hosted results. Commit and push separately under M34A-10AB.
