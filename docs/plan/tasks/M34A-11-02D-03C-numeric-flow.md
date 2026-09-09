# M34A-11-02D-03C — numeric flow

- Status: complete
- Depends on: M34A-11-02D-03B

## Goal

Convergent actual-graph range/relational flow with effect invalidation and point-local safety checking.

## Definition of done

03B closes at 6c74c36 after clean independent review and the full cached
container gates. Implement this slice in the following dependency order:

1. Add private flow domains composed from checked 03A ranges: normalized finite
   integer interval unions (so nonzero guards retain the hole at zero), floating
   ordered/NaN restrictions, intersection and conservative widening.
2. Derive numeric storage roots/selectors and address exposure from actual AST;
   implement scope exit, direct/indirect writes, joins and opaque-call invalidation.
3. Evaluate actual expression/initializer/place operands with guarded
   short-circuit/conditional paths and closed typed predicate refinement.
   Record actual arithmetic/conversion provenance and unresolved storage/index
   obligations, not input-authored safety flags.
4. Solve the immutable graph, widening growing loop inputs conservatively,
   then strictly check every evaluated operation at converged incoming states.
   No intermediate optimistic result or failed evaluation may prune a path.
5. Consume 03B's actual per-point step phase to distinguish counter<bound before
   the step from counter<=bound afterwards; derive materialized known-call
   predicate relations without assuming nonzero Int results are one.
6. Prove the combined guard/staleness/effect/loop/extent matrix, private fact
   boundary and full cached gates; obtain independent uncapped review and
   close parent03 only after its complete checklist is covered.

Production modules separate flow domains, storage identities, state,
expression evaluation, predicate refinement, graph solving and obligations.
No module may become a catch-all emitter or take arbitrary source strings.
Unknown generated-call return values use their actual type domains; later
call summaries cannot be guessed from a matching prototype or function name.

- Private authenticated numeric storage keys/state; edge refinement uses typed polarity/cases/loop identity.
- Conservative monotone joins/widening, no timeout-as-success or giant loop enumeration.
- All evaluated expression children checked at converged inputs; short-circuit and conditional guard-local refinement.
- Writes and opaque effects invalidate aliases/globals while preserving never-address-taken local storage facts.
- Use actual proved loop evidence for counter<bound and safe increment; maintain nonwrapping-size/index obligations for storage/call composition.
- Known/generated calls contribute only sound return ranges with outstanding safety obligations; no summary is manufactured.
- Compose with ContextFacts and constants; dynamic pointer validity/ownership still 04/05.

The detailed numeric contract is in
[numeric range proof](../../specification/typed-generation/languages/c/numeric-range-proof.md).
The exact loop form remains [counted loops](../../specification/typed-generation/languages/c/counted-loops.md).
These slices implement the parent task; none weakens its complete-stage gate.

## Tests and proof

- Guard reversal/nondominance/staleness, joins, loops and unknown parameters; arithmetic bounds before execution.
- Allocation products distinguish numeric modulo from checked extent, with wrong/missing preguard controls.
- Zero-bound dead paths vs unknown iteration, short-circuit division guards and float NaN guards.
- Full cached tracked/release/lint/eight-target plus final independent review; parent03 closure checklist.

## Commit gate

Record focused and full cached Linux Dev Container evidence and an independent
uncapped Sol Extra High review. Commit and push M34A-11-02D-03C separately. Keep
parent 02D-03 open until every slice and its combined checklist pass. No partial
numeric or loop fact is a C safety, allocation or rendering certificate.

## Implementation and current evidence

Private normalized integer unions retain nonzero holes; floating restrictions
retain NaN classes and zero signs. Typed storage roots/selectors, actual edge
predicates and immutable loop phases feed a widening worklist. A separate strict
pass checks every evaluated operation at converged inputs. Algebraic unsigned
size guards prove variable addition/products without independent-interval
approximations; they retain actual guard witnesses and invalidate on effects.

NumericFacts borrows the same ContextFacts and stores actual incoming states,
Calculation/Index/Call obligations and WriteBytes product facts. Site validation
checks actual pointer-identical occurrences inside the owning graph action.
Re-deriving a materialized predicate uses a non-executing proof mode, so an old
operand cannot acquire a fabricated runtime site at a later condition.

Closed unresolved-origin variants retain actual arithmetic, aggregate, read,
write and call references. Root fallbacks cover unseen fields. Aggregate reads
copy/rebase source snapshots; unknown sources/effects remain unproved. Numeric
library transforms preserve operand history; classifier values are distinct.
No caller-authored range, safe flag, target source text or renderer path is added.

The current C suite has 317 unit tests. New focused modules cover finite domain
oracles, guards and inversions, NaNs and signed zeros, modulo conversions,
pre-operation size bounds and algebraic relations, stale/effect/branch joins,
member and array identities, implicit call products, promoted switch cases,
actual private sites, and loop consumers across branch-local steps, joins,
Continue and nested cycles through SIZE_MAX without unrolling native loops.

Passing cached Linux Dev Container evidence on the repaired frozen source:

- Five focused targets (C units, Rustdoc, Clippy, Buildifier, docs):
  `98bfead3-bf2e-474b-9153-769cb723e5ae`.
- Full tracked graph: `a43cd842-e8fe-4797-924a-8d1629f818ce`
  (445 rules, 320 test targets pass, 45 executed).
- Release: `eb1d146d-c7cc-4958-aa34-182be6a4a172`
  (257 test targets pass).
- Eight-target conformance: `4f6d53aa-c682-4115-81e5-bbca87b28b63`
  (50 cases and one portable test agree with the evaluator; repeated manifests
  are byte-identical).

These earlier gates precede the final repairs and closure evidence below.
The user-owned untracked stdlib-abs example is excluded from this checkpoint.

## Review findings and disposition

The first uncapped Sol Extra High review, c17_contextual_final_review, audited
the full new numeric flow and its existing domain/control prerequisites from
base 6c74c36665503118dc668dd5fa880b578b498a84. An existing reviewer context was
used for a new scope, not described as a newly spawned blind review.

Two substantive findings were accepted and repaired:

1. Aggregate assignment/initialization and indirect, variable-index or opaque
   writes could lose arithmetic history, admitting a wrapped extent as clean.
   Actual field snapshots now project/rebase; unresolved writes/calls/read
   sources remain typed origins, including fallbacks for unseen cells.
2. FloatTruncate/FloatRemainder created fresh clean numeric results without
   argument lineage. They now inherit it. IsNan/SignBit remain classifier results.

The reviewer also identified required proof gaps for Call/Index sites and the
numeric consumer of branched/nested/Continue loop phases. The dedicated site and
loop-path tests now cover those handoffs, including negative post-step consumers.
Root added strict actual-action site validation and non-executing predicate
re-derivation while closing the site gap.

A tentative signed-zero finding was withdrawn, and root independently agrees:
restrict_float constructs an Ordered zero range, not Exact(+0). Intersection
therefore uses numeric bounds and preserves the original -0; a later exact
intersection compares that same sign. No production change was justified.
A focused opposite-sign equality/negated-inequality branch test localizes this
already-correct behavior.

The lineage regressions first ran against unchanged production at
`b2b16959-b6fb-416b-b61c-bc05089cb849`: all five test functions failed because
the checker returned Ok instead of UnprovedSizeArithmetic. Clean-copy and
clean-transform controls are retained. After repair, focused gates
`003a628f-84d3-436b-86c4-3c1255bb0b3a` and
`ab02c7ee-6ca1-41a8-8de5-880accf8a1a0` passed, followed by the complete
312-test focused/full evidence above. A separate existing Sol Extra High
reviewer context, c17_contextual_repair_review, completed the repaired-source
audit and found one further defect: a missing global cell on one predecessor
was materialized as clean by a branch join. Root accepted this finding; the
four-way branch-write regression reproduced red at
`6d7da6f1-b287-4b86-aac1-0f981d27082a` (the single-write case returned Ok).
Absent global values now intrinsically retain their authenticated object origin
in State::number, including joins and guard refinement. Read-time absence
detection is removed. Both predecessors assigning clean values remains positive.
No further required findings or proof gaps were reported in that full audit.

Passing cached gates after the global repair:

- Five focused targets, including all 313 C units:
  `8df7d6ed-ed26-4168-b867-9de7b793bf74`.
- Full tracked graph (445 rules, 320 tests, 45 executed):
  `8482f651-bf73-4502-8af7-0d3df8c6d7a1`.
- Release (257 tests): `dc722d8a-a58b-4bd1-bbf2-71b366f8752c`.
- Deterministic eight-target conformance (50 cases plus one portable test):
  `e5521189-8ecc-442f-9d94-69961c761114`.

The existing Sol Extra High context c17_ast_construction_review completed a
new independent full-scope audit. It found one over-rejection defect and two
required proof gaps, all accepted by root:

- The shared walker's Address callback also traverses non-reading member/array
  bases. Treating it as address exposure incorrectly poisoned never-address-taken
  aggregates across opaque calls. Exposure now comes from actual AddressOf
  expression nodes; dependency traversal and unconditional global exposure stay
  intact. Member/array positives first failed red at
  `5db80b3e-32a4-4dde-b55a-463c9982d3e6`; explicit subobject-address negatives
  remain. This defect rejected safe input; it did not admit unsafe input.
- Generated numeric returns lacked direct proof controls. Both direct and
  indirect Size returns now reject as allocation extents without a body summary;
  comparison and Bool classifier results remain positive independent values.
- Index obligations lacked a direct lineage-retention control. The internal
  test now checks pointer-identical arithmetic origin on a potentially wrapped
  index and no origin on an exact clean index. Extent discharge remains 04.

No other production defect survived that full audit. It read the full scoped
implementation, prerequisites and test matrix without edits or builds. A new
repair review in the existing c17_contextual_repair_review context checked the
changed surfaces and their dependencies against its earlier full audit. Its
final verdict was clean: no remaining substantiated defect or required proof
gap. This was not a fresh blind spawn. Production/tests/specification remained
frozen during the repair review and final gates. All accepted findings have
explicit regression controls; no optional expansion was made a closure condition.
The final evidence closes 03C and parent 03, not storage/call or renderer safety.

Passing cached gates on the final repaired frozen source (317 C units):

- Focused five targets: `39476103-f140-41ae-be06-df14a0672da1`.
- Full tracked graph, 445 rules and 320 tests, 45 executed:
  `9e57e76f-6c8e-44f7-8592-b86e93e025d7`.
- Release, 257 tests: `67372068-dafa-4bd7-9fd2-696d18133499`.
- Deterministic eight-target conformance, 50 cases plus one portable test:
  `a01c5ca6-66ba-4de9-9b7a-1b314db75544`.

## Earlier development runs

Earlier passing focused checkpoints were bf9e406a (282 C units), df0b075f,
and 9d421063 (298 C units plus all five focused targets). Before the review
repairs, full tracked/release/conformance gates also passed at 72ef1b66,
983c495a and 3e3ea798. Those runs did not establish absence of the later-found
lineage defects and are not substituted for the repaired-source evidence.

Non-passing development invocations remain explicitly non-passing:

- 7d90f30b: internal DomainTransfer/NumericLoss re-export visibility corrected.
- bd085b01: Clippy required a boxed selector payload, collapsed conditional and
  grouped floating-comparison operands.
- 389def03: a test condition omitted the explicit Int-to-Bool conversion.
- 72847e26: Clippy required a boxed relation term and direct state initialization.
- f77ce991: the independent test oracle switched to checked_div as linted.
- 6cea2e28: a stale Number import was removed after provenance refactoring.

No lint suppression, cache disabling, weakened diagnostic or false successful
invocation claim was used to close those failures.
