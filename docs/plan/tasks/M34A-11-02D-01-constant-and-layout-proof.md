# M34A-11-02D-01 — Exact C constants and object layouts

- Status: complete
- Depends on: M34A-11-02D-00

## Goal

Replace constant-expression category alone with exact checked values and layout
facts, using the frozen Linux x86_64 ABI rather than host arithmetic assumptions.

## Definition of done

The exact model and evidence boundary are specified in
[constant/layout proof](../../specification/typed-generation/languages/c/constant-and-layout-proof.md).
02D-00 closes at b8764957bb49966ccbf6414b49815db7d25f21d9 after independent
review and all cached gates. Implement this slice in the following order:

1. Extend the authoritative scalar model with closed representation/width
   categories; add private checked integer/numeric values and typed diagnostics.
2. Derive immutable object layouts from registered types/members with checked
   padding/products and explicit traversal state; no host sizeof assumptions.
3. Evaluate actual constant trees, promotions, conversions and short-circuit
   selection; use distinct type-form and evaluated-operation walks.
4. Compose context, all-syntax layout and static initializer/assertion checks
   over the same borrowed package; expose no source-validity certificate.
5. Add focused boundary/mutation matrices and independent two-compiler probes,
   permanent release membership, all cached gates and an uncapped review.

Keep representation, values, conversions, integer operators, floating operators,
tree evaluation, layout traversal and package integration in cohesive modules.
Do not put this whole slice in one verifier or duplicate scalar promotion rules.

- Implement private checked scalar values, integer promotions/conversions and
  layout size/alignment/padding calculations from authoritative registered types.
- Traverse typedefs and complete aggregates; reject recursive by-value layouts,
  incomplete SizeOf/AlignOf targets and checked size/alignment/array overflow.
- Evaluate all admitted integer constant operators with target-width semantics;
  reject signed overflow, division/remainder zero and minimum/-1, invalid shifts
  and unproved narrowing. Unsigned wrapping follows its actual promoted type.
- Re-evaluate static initializers and assertions from actual AST children;
  require assertion truth. Respect conditional evaluation without skipping
  structural checks of unselected syntax.
- Treat opaque FILE distinctly from complete MaxAlign. Values, identities and
  errors are typed enums/wrappers, never integer proof labels or safe flags.
- Expose only crate-private immutable facts for later analysis/resources.
  Compiler resource budgets and native frame accounting remain stage 04.

## Tests and proof

- Every scalar width/rank, signed extrema, unsigned wrap and usual-conversion
  pair; zero divisors, minimum/-1, shift boundaries and narrowing mutations.
- SizeOf/AlignOf for scalars/pointers/enums/aliases/arrays/structs/unions, padding,
  mutual cycles, incomplete objects, huge checked products and alignment sums.
- True/false assertions, invalid constant children and skipped-invalid-operation
  controls paired with unconditionally invalid structural-child rejection.
- Independent pinned native ABI/constant controls, focused Rust/compile-fail/lint,
  full cached tracked/release/eight-target gates.

## Commit gate

Record exact passing evidence, then commit/push M34A-11-02D-01. A constant or
layout fact is not a whole-program safety or rendering certificate.

## Implementation and evidence

The private ownership modules now derive checked numeric values and layouts
from the same borrowed registry/files checked by the contextual layer. Scalar
representations are closed enums; integer conversions/operators, known values,
tree evaluation, layout traversal and package integration are separate modules.
All-syntax type/layout checks precede selected constant evaluation. No public
facts constructor or rendering certificate is introduced.

Five focused test modules bring the C unit suite to 201 tests. Independent
constant/layout probes run under pinned Zig at O0/O2 and GCC 14.2; all three
targets are permanent release-gate members. The native fixture has an exact
source-policy exception with adjacent-path rejection controls. Production
modules added by this slice remain at most 233 physical lines.

Passing cached Linux Dev Container evidence on the reviewed worktree:

- Focused C units, Clippy and Buildifier:
  `667f60e9-fca4-4333-a9ff-5ae7ed154a65` (all three targets pass).
- Full tracked gate: `331fcb69-1dcb-4e24-a352-ea1c7cd22668`
  (445 rules; all 320 test targets pass, 48 executed).
- Release gate: `26c88f5e-2101-424f-873c-5892f4a28e5c`
  (all 257 test targets pass).
- Eight-target conformance: `18316b12-0f7e-46f9-b19b-5742e1c7bc80`
  (50 cases and one portable test agree with the evaluator; repeated output
  manifests are byte-identical).

Earlier invocation `b0f9216b-0814-4e7b-ab48-1f2d89328295` passed the C,
rustdoc, native and Clippy targets but failed Buildifier on release-label
ordering. The ordering was corrected before the passing gates above; that
earlier invocation is not claimed as an overall pass.

Independent uncapped Sol Extra High review by c17_contextual_final_review
completed with no actionable production defect or required semantic proof gap.
The reviewer audited the full production boundary, test matrices and native/
release/source-policy wiring without rerunning tools or modifying the tree.
This is an independent new review scope using an existing reviewer context,
not a claim that a new agent was spawned. The optional suggestion to add a
package smoke case for every evaluator leaf/static-address arm is not a closure
requirement: current core and composition matrices cover this slice's contract;
the exhaustive grammar-variant inventory remains a mandatory stage 04 gate.
That suggestion does not identify an unchecked operation or an unsound fact.

Documentation and Buildifier also pass at
`360546e2-cf12-44fe-a5e1-b63a4e267698`. The base checkpoint b876495 has a
successful hosted CI run, 34290668049; that result is not attributed to this
checkpoint's later commit. Local gates and independent review close this slice.
C migration, runtime safety and the final exact-SHA hosted gate remain open.
