# M34A-10Y — Bind strategy certificates to executable Java mappings

- Status: complete
- Depends on: M34A-08V and M34A-10U
- Blocks: completion of M34A-10W, M34A-10R, and M34A-11

## Completion (2026-09-08)

Implemented and integration-verified at
`708c37bcda25c8e518eb241ee89bd13b19075226`. The complete local gates, all
eight hosted CI jobs and fresh uncapped Sol Extra High review pass with no
remaining demonstrated core error. See the current proof in
[M34A-10](M34A-10-java.md) and the final review disposition in
[M34A-10R](M34A-10R-java-review-remediation.md). Java is ready for user design
review; user acceptance is not implied. Earlier checkpoint/open-status notes
below are retained historical evidence, superseded by this completion record.

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

Exact admission was implemented in the first checkpoint; the next section records
the subsequent checked-invocation implementation.
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

## Stage 2 implementation and working-tree review

All 42 registered capability mappings now require an associated output-typed
plan and a no-default selector. The builder automatically stores the checked
wrapper, selects before lowering on the same supplied instance, and rejects
actual-output mismatches with diagnostics. Every capability owns a small plan
module; shared helpers are closed skeletons, not a central strategy classifier.
Intrinsic `Direct` was renamed `Infallible` to avoid mislabelling runtime calls.

The broad Sol Extra High working-tree review found eight accepted core contract
gaps, all repaired with independent mutations:

1. Preserve native/payload enum type shape in the plan input.
2. Reject non-orderable Java representations instead of a native fallback.
3. Authenticate structured Boolean owned-node types.
4. Authenticate portable-test helper result types.
5. Authenticate uninhabited-interface throw construction and typed message.
6. Authenticate shared known-constructor owner/signature/argument consistency.
7. Authenticate generated assertion message contents, not only string type.
8. Authenticate generated file source attribution alongside its other root fields.

The compiler proof uses the actual production trait declarations and requires a
positive control before accepting E0046/E0271 negative cases. The reviewer
confirmed this proves the scoped required-plan contract, not arbitrary lowering
correctness; a more syntax-aware test extractor was optional, not a core error.
The last two metadata/literal findings complete the owned-root contract but were
not claims that the earlier output failed Java compilation. Operand subtrees
remain deliberate holes, with final Java verification independent of certificates.

The final broad working-tree re-review found no remaining actionable stage-2
core errors. All eight findings have focused mutations, including content-only
assertion and source-only file changes. The reviewer did not run tests; the
following independent executions establish the final local proof:

- Full tracked graph: 435 analyzed rule targets, 310/310 tests pass (invocation
  `e8ead380-528f-42ff-bd5a-6de986860e02`). Only the untouched user-owned untracked
  `examples/real-world/stdlib-abs/` package is excluded.
- Explicit release gate: 247/247 pass (invocation
  `2dfb357e-5004-44a2-8c32-75836ed5fe6b`). Rustfmt, strict Clippy, Buildifier,
  source policies, snapshots, native Java and all historical ports are included.
- Deterministic conformance: 50 cases and one portable test agree between the
  evaluator and all eight targets; repeated manifests are byte-identical
  (invocation `23a5ff76-dc8c-4178-8ab9-5d2880a20f31`).
- Linux Cargo 1.98 compatibility: 158 Java unit tests and eight doctests pass,
  including actual Java 21 consumer compilation outside Bazel's test environment.
- The complete gate initially identified the handwritten Python compiler driver
  as a generated template. A path-exact infrastructure exception fixes that
  classification; permanent injections still reject adjacent/copied templates.

Caches remain enabled per the current CI/local policy. Hosted CI and the final
fresh immutable-checkpoint review remain required before closing this task.

## Immutable-checkpoint integration cleanup

Checkpoint `edada37a1ae11a03cfbb227c6fd80e7d4d2bce3f` was committed, pushed,
and matched against the remote main ref. The root integration audit and fresh
review identified a module-layout regression: 16 new plan modules and four
preflight modules used parent wildcard imports. The unqualified explicit-import
rule applies to them too; no exemption was introduced. All 20 production sites
now name their real dependencies, the preflight root no longer aggregates
child-only imports, and affected preflight tests import their own dependencies.
The focused Java/Rustfmt/Clippy/Buildifier/layout gate passes 32/32 after this
cleanup. The complete import-only checkpoint replay also passes 310/310 tracked
tests, 247/247 release tests, and 50+1 deterministic evaluator/eight-target cases.

The same fresh review demonstrated a separate existing target-verifier hole:
numeric wrapper casts incorrectly admitted unboxing followed by narrowing. A
new independent 36-pair AST matrix first failed on `Byte -> Char` against the
unrepaired implementation. The production relation is now exhaustive by boxed
primitive; all 15 admitted pairs also enter the real certified renderer/Java 21
compiler oracle. A native negative fixture retains `Long -> int`, `Integer ->
byte`, and `Byte -> char` counterexamples. This finding is accepted as a core
target-validity defect, not an optional feature.

Final local repair proof: 435 analyzed rules / 310 tests pass (invocation
`4255b202-5803-4725-b83f-5b414cd6ea5b`), release 247/247 passes
(`40bed654-9d0f-4e89-ace1-2a161ed7275d`), and deterministic conformance passes
50 cases plus one portable test across the evaluator and all eight targets
(`e34767bc-e138-4da4-b7ee-3d3b9a01e194`). Linux Cargo 1.98 passes 159 Java unit
tests and eight doctests. Hosted run `34176018075` is green in all eight jobs
for the preceding `edada37` checkpoint, not yet for these follow-up repairs.
The current review's final conclusion and a fresh repaired-checkpoint review
remain required; this task is still in-progress.
