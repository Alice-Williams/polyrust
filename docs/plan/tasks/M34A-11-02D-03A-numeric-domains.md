# M34A-11-02D-03A — numeric domains

- Status: complete
- Depends on: M34A-11-02D-02

## Goal

Private checked scalar abstract domains and operation transfer relations, without claiming control-flow or allocation safety.

## Definition of done

- Closed typed integer/binary64 range categories and checked constructors from actual scalar model.
- Promotion/conversion/operator transfer with sound intervals and undefined-operation diagnostics; reuse exact constants for singleton behavior.
- Represent range imprecision and unsigned modulo behavior distinctly from proof of nonwrapping size arithmetic.
- Float special values/exclusive integer conversion boundaries, no invented normalized known-predicate results.
- No public fact builders; numeric modules cohesive below size policy.
- Connect exact constant-tree operations/conversions to the kernel while keeping
  the exact-value model independent for singleton and finite-property oracles.

The detailed numeric contract is in
[numeric range proof](../../specification/typed-generation/languages/c/numeric-range-proof.md).
The exact loop form remains [counted loops](../../specification/typed-generation/languages/c/counted-loops.md).
These slices implement the parent task; none weakens its complete-stage gate.

## Tests and proof

- Independent finite exhaustive small-domain enumeration of exact evaluated values vs interval containment.
- Every scalar pair/operator legality, signed extrema, min/-1, zero, shifts, unsigned wrap and narrowing.
- NaN/Inf/signed zero, fractional truncation and exact power-of-two exclusive cast bounds.
- Rustdoc/private boundary, lint and full cached gates plus independent review.

## Commit gate

Record focused and full cached Linux Dev Container evidence and an independent
uncapped Sol Extra High review. Commit and push M34A-11-02D-03A separately. Keep
parent 02D-03 open until every slice and its combined checklist pass. No partial
numeric or loop fact is a C safety, allocation or rendering certificate.

## Implementation and evidence

The private numeric kernel separates checked integer and binary64 domains,
conversions, integer operators, shifts and transfer classification into cohesive
modules. Bounds come from the authoritative scalar ABI. Exact constant-tree
operations now consume this kernel; CNumber remains the independent exact
oracle for singleton and finite exhaustive containment tests. NonWrapping and
MayWrap describe one primitive operation after C input conversions, not an
entire expression, allocation-size proof or finite floating-point guarantee.

Four focused test modules exercise all 13 scalar types, 18 binary operators,
unary operations and conversions, mixed extrema, finite exhaustive interval
containment, useful bounded results, unsigned wrap, NaN/infinity/signed zero
and exact exclusive floating-to-integer boundaries. Private-domain compile-fail
controls are included. The C unit suite contains 231 tests.

Passing cached Linux Dev Container evidence on the reviewed worktree:

- Focused C units, Clippy, Buildifier and documentation:
  `d4342e22-c133-49af-b95d-52b057a8c900`.
- Final full tracked gate: `795447a9-8992-41be-9bf0-f6c3136086a0`
  (445 rules; all 320 test targets pass, 47 executed, including Rustdoc).
- Release gate: `3191aa06-6da9-4fc0-a507-303383426be9`
  (all 257 test targets pass).
- Eight-target conformance: `f1bcd2a3-f87c-4cde-886b-0024bc24340d`
  (50 cases and one portable test agree with the evaluator; repeated manifests
  are byte-identical).

Earlier invocation `8e633409-57cd-4219-8e18-cc0b071189f8` failed Clippy because
test-only containment helpers had no consumers yet; the property tests now use
them without warning suppression. Invocation
`0a250ef4-83d7-4db6-90ab-4f1897f0dc5a` passed 230 of 231 units and the other
focused targets: one test expected Size instead of the authoritative U64
arithmetic result identity. Only the test expectation was corrected. Neither
invocation is claimed as an overall pass.

Independent uncapped Sol Extra High review by c17_contextual_repair_review
completed PASS with no substantive production error or required proof gap.
This was a new review scope using an existing reviewer context, not a newly
spawned agent. Source remained frozen throughout review and final gates.
The reviewer inspected every scoped production/test file and the complete
numeric proof matrix without editing or running builds.

One tentative NaN-join finding was independently rejected and then withdrawn:
containment handles every NaN through may_nan before comparing exact bits.
Canonicalizing a NaN-only join therefore preserves all numeric observations;
the specification deliberately does not promise NaN payload identity. A direct
two-payload regression is optional localization, not a missing correctness
obligation. No production change was made for that withdrawn finding.

Local gates and independent review close this slice. Parent 02D-03 remains
open for counted-loop and numeric-flow proofs. This checkpoint does not claim
hosted CI success for a commit that has not yet been created.
