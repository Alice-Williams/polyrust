# M34A-10Y — Bind strategy certificates to executable Java mappings

- Status: in-progress
- Depends on: M34A-08V and M34A-10U
- Blocks: completion of M34A-10W, M34A-10R, and M34A-11

## Goal

Make each exact dynamic support decision derive from the registered mapping
which performs that operation, not an independent central strategy switch.

## Implementation order

1. Replace premature strategy decisions with exact Java-local admission:
   feature use, closed capability owner (or explicit structural reason), and
   required registered slots. Validate against the lowerer's actual mapping set.
   Reject legacy zero-variant enums before lowering. Do not change the generic
   collector to encode target AST choices.
2. Move operation strategy selection into the owning mapping contract. Use
   closed typed operation/shape inputs; registration supplies the same mapping
   instance to certification and lowering. No blanket or default strategy.
3. Preserve shape facts needed to distinguish native operators, runtime
   helpers, and structured expression plans. In particular, boolean RHS plans
   containing statements require structured short-circuit control flow.
4. Make the Java builder automatically store a checked executable wrapper.
   Every mapping supplies a required, no-default input-plan selector. The wrapper
   selects before lowering and checks the actual output against the selected
   mapping-owned root skeleton. Direct operand subtrees remain opaque holes;
   runtime/structured roots authenticate only the nodes owned by that mapping.
   Use sealed associated Plan types with exact Output equality and required
   representation/verification methods. Capability-owned closed enums may reuse
   closed root skeletons; one central mega-enum is not required. Every mapping
   retains Vec<Diagnostic> errors so wrapper rejection remains lossless.
5. Remove the central blanket unary/binary strategy switch. Cover every
   non-intrinsic input/output variant too, including erased aliases. Rename
   the infallible intrinsic wrapper so it cannot be confused with native code.
6. Run the full proof and fresh review, then close the Java integration gates.

## Definition of done

- Native wrapping arithmetic, bitwise/float operators, boolean not, and string
  concatenation no longer receive runtime-helper certificates.
- Payload-free enum equality selects `Enums`; local reads select the capability
  owning their checked binding origin. Both expression and constant paths agree.
- Admission use/owner/prerequisite mismatches are rejected independently of
  invocation plan/output mismatches. Recomputing a central strategy switch is
  not proof; exact strategy is selected only once the closed Java input exists.
- Removing a mapping prevents both typed admission and dynamic certification.
- New implementations and tests remain in small capability-owned modules.

## Tests

- Table-driven coverage of every closed intrinsic operation and relevant shape
  against the actual typed AST/plan produced by its owning mapping.
- Short-circuit RHS fixtures with and without prerequisite statements; native
  execution proves RHS evaluation remains conditional.
- Enum/general equality and all four local binding-origin certificates.
- Zero-variant enums on the legacy dynamic path: the shared shape collector
  and Java currently disagree on whether they count as payload-free. Resolve
  representation or reject the shape explicitly; never certify a different
  owner from the one invoked by lowering. This is not a typed enum constructor.
- Deliberately mismatched strategy/output witness is rejected.
- Automatic-wrapper, missing-plan compile-failure, nested runtime operand,
  left-only prerequisite, and every non-intrinsic node-category regressions.
- Full Java tests, Rustfmt, strict Clippy, Buildifier, tracked Bazel graph,
  release gate, deterministic conformance, hosted CI, and fresh uncapped review.

## Commit gate

Commit and push after local proof, citing this task. M34A-10W remains open until
this task and M34A-10X both meet their exit criteria.

## Design review decision

The uncapped Sol Extra High design review confirmed the two-stage design.
FeatureUse cannot know a lowered Boolean RHS statement plan or an operand's
Java representation. A recursive output-only classifier is also ambiguous:
`!SemanticEqual(...)` can be Equality's runtime negation or BooleanLogic's
direct negation around an opaque runtime-produced operand. The normative
[Java certificate contract](../../specification/typed-generation/languages/java/strategy-certificates.md)
therefore requires exact pre-lowering admission and plan-guided invocation
verification, not a fictitious preflight native/emulated decision.

## Stage 1 implementation and review

Exact admission is implemented; checked invocation plans remain open.
JavaLoweringStrategy and the central Native/Emulated strategy switch are gone.
The single shared capability catalogue now supplies enum-valued CapabilityId
identities alongside compile-time slot indices. Admission records exact checked
uses, mapping/structural owners, and prerequisites; validation reads the actual
mapping set copied into the lowerer, not a newly constructed default plugin.
Legacy zero-variant enums fail preflight with attributed diagnostics.

The fresh uncapped Sol Extra High review accepted three concrete follow-ups:

1. Structural enum matches initially omitted the general-pattern mapping.
   Admission and lowering now share checked match dispatch. Each deduplicated
   feature/shape carries the union of actual selected slots across represented
   occurrences. Native/payload enums, with/without wildcard arms, have both
   missing/wrong-prerequisite mutations and separately compiled Java consumers.
   The wildcard fixture also exposed a legacy payload-free pattern indexing
   the payload-only variant table; it now invokes native enum equality.
2. A genuinely empty module has no FeatureUse, but still invokes Modules.
   Program-level prerequisites are now independent of the feature vector.
3. Java also always invokes PortableTests for its generated harness. That slot
   is an explicit backend service prerequisite, including empty programs.
   Generic semantic requirement inference is intentionally unchanged: no user
   test exists in the empty program, while construction of JavaPlugin and
   JavaLowerer already requires the complete concrete mapping set. The reviewer
   confirmed there is no safe typed path to a Java lowerer missing this service.

The final scoped re-review found no remaining actionable stage-1 or CI-harness
defect. Real empty dynamic/typed programs assert the exact invocation ledger;
admission tests independently mutate feature use, owner, prerequisites, and
program-level service inventory. The focused Java/strict-lint gate passes all
30 targets. Final checkpoint proof: 433 tracked rule targets / 309 tests pass,
the explicit release gate passes 246/246, and deterministic conformance passes
50 cases plus one portable test across the evaluator and all eight targets.
None of this closes stage 2 or the overall Java integration milestone.
