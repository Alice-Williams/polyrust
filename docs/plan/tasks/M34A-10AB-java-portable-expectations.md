# M34A-10AB — Reconcile Java portable expectation comparison

- Status: complete
- Depends on: M34A-11-01R

## Goal

Repair the concrete comparator mismatch discovered during the C design review,
without changing established portable floating-point semantics.

## Definition of done

- Keep IEEE semantic equality, NaN-class portable expectation equality, and
  raw-bit representation equality separate and explicitly named in typed APIs.
- PortableTests uses expectation comparison for ordinary success values, including Result Err payloads;
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
  nested success/Result Err payloads, and enforce matching completion counts.
- Computational error expectations remain exact string error-code assertions;
  they contain no F64 payload and are not confused with ordinary Result Err values.
- Negative controls: finite mismatch, NaN versus finite, signed-zero mismatch;
  raw-bit audit still rejects different NaN payloads.
- Rust/mapping/compiler tests, Java 21 lint, native public consumers, resource
  reservations, all tracked Bazel rules, release gate, eight-target conformance,
  hosted CI and fresh Sol Extra High review. Normal caches stay enabled.

## Commit gate

Record exact failing controls, successful invocations, review dispositions and
immutable hosted results. Commit and push separately under M34A-10AB.

## Implementation evidence (2026-09-08)

- Pre-fix `65719ba6-a340-49b8-a9d2-5dd4b426e484`: the evaluator accepted all
  seven NaN-class cases; both GeneratedTest and ConformanceTest independently
  failed with a value-mismatch assertion. The cases cover scalar, list,
  option, Result Ok/Err, nested record and legacy payload enum.
- A closed EqualityKind now binds the dispatcher callable, value member and
  float rule. Semantic comparison remains IEEE; expectation comparison accepts
  two NaNs or identical non-NaN bits. Structural AST references derive all
  dependencies. No production third-party dependency was added; the evaluator
  dependency is test-only.
- Three regression tests run both native entry points, reject four finite/
  NaN/signed-zero mismatches with exact case diagnostics, and separately compile
  an external public consumer for IEEE equality and exact literal/FloatAbs
  payload/sign assertions. The initial consumer attempted private helper calls;
  that test setup was corrected to public generated functions before evidence
  was accepted. Both harnesses enforce their exact completion inventory.
- The first full gate passed 310/311 tests, with only the intended curated
  snapshot drift remaining. Both snapshots were refreshed from Bazel output,
  not hand-authored replacements.
- `44aa8b53-62d3-4a9f-9f24-06de043920ee`: all 436 tracked rules and all
  311 tests pass, including Rustfmt, Clippy, Buildifier, Java/native/compiler,
  policy, resource and byte-for-byte snapshot tests, with normal caching.
- `ca2bbefe-dd25-4d86-a435-82629ce17c62`: all 248 release tests pass.
- `4bcbbe96-165f-4a08-917f-262c5fe8d3ee`: 50 cases and one portable test,
  evaluator/eight-target agreement and repeated manifest determinism pass.
- Supplementary Linux pinned Cargo compatibility passes: Java 208 unit tests
  and doctests; checker 29 unit tests and one doctest. Bazel remains authoritative.

## Independent review and hosted closure

A fresh uncapped Sol Extra High read-only review of immutable
`74182bd44127eb84f3ca9e57cc53f0f2ee7a3456` found no remaining core correctness
or specification defects. Root independently checked its reasoning: the three
comparison contracts remain distinct, all recursive expectation paths use the
correct comparator, both native harnesses enforce their inventory, and the
evaluator dependency is test-only. The optional suggestion to replace paired
internal record/tagged-helper parameters with EqualityKind is deferred: every
trusted call is consistently paired, the dispatcher already binds the enum,
and no failing construction or behavior was demonstrated. This is additional
hardening, not an unresolved defect.

All eight hosted jobs pass for that exact commit in
[run 34223622371](https://github.com/Alice-Williams/polyrust/actions/runs/34223622371),
including the cached release gate. This descendant also verifies the shared
recursive-equality repair `c260c8b`; its superseded run is not claimed green.
The fresh shared-repair review independently found no core defects. Together
with the local evidence above, this restores Java compliance and closes AB.
