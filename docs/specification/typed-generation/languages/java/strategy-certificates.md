# Java admission and mapping certificates

- Status: normative for M34A-10Y; implementation in progress

## Stage 1: exact admission

Java preflight MUST record each exact checked FeatureUse, its closed capability
owner, and its prerequisite slots. Modules and the synthetic PortableTests
harness are Java program-level prerequisites even when Core has no feature
uses. PortableTests here is a backend service dependency: the fixed concrete
Java mapping set requires it independently of the generic typed program's
semantic requirements. An empty generic program does not acquire a fictitious
user-test requirement merely because Java emits an empty native harness.
Calls and fallible intrinsics also require ResultPropagation. Payload-free enum
equality belongs to Enums, not general Equality. Local reads belong to Functions,
LocalBindings, Loops, or PatternMatching according to their checked origin.

Structural assembly (blocks, evaluation, ownership sequencing, literal dispatch,
and shape-dependent match orchestration) MUST be represented honestly as a
closed structural admission, not a fictitious Modules lowering strategy.
Actual literal/type and pattern mappings remain separately required by the
collected type/pattern uses. Shape-dependent match admissions MUST additionally
carry the actual enum/pattern mapping slots selected from checked Core.
Because the shared collector deduplicates feature/shape uses, prerequisites
cover the union of checked occurrences represented by that shape.
Legacy zero-variant enums MUST fail preflight until
an explicit representation is implemented; typed enums remain nonempty.

Admission MUST be validated against the same mapping set held by the lowerer.
It MUST NOT guess Native/Emulated before Java input plans exist. Java may use a
local admission vector rather than the generic early-strategy SelectedFeature.

## Stage 2: mandatory checked invocation

Every sealed Java mapping MUST implement a no-default plan selector accepting
its exact borrowed Input. A sealed family of associated JavaMappingPlan types
combines representation and mapping-owned output skeletons. Each mapping's
Plan MUST have Output equal to that mapping's Output. Capability-owned closed
enums may reuse closed root skeletons without creating one giant enum file;
their required selectors/verifiers MUST not admit a blanket or wildcard plan.
All mapping errors remain Vec<Diagnostic>, so wrapper mismatches are ordinary
attributed diagnostics, never panics or erased errors.
JavaPluginBuilder::support(M) MUST store
CheckedJavaMapping<M> automatically. The stored wrapper MUST:

1. select the plan before consuming input;
2. invoke the supplied mapping instance;
3. verify the actual typed AST/plan against that selected skeleton;
4. reject a mismatch with diagnostics before returning output.

Supports<C> returns that checked executable wrapper around the supplied mapping.
Raw unwrapped handlers MUST NOT be stored in production plugin slots. A checked
wrapper is not itself an admissible raw handler, preventing double wrapping.

## Representation and owned skeletons

Representation describes the outer mechanism owned by a mapping, not every
nested dependency. Direct unary/binary operands are opaque input holes. Runtime
negation checks the owned unary node and its exact runtime callable. Structured
control takes precedence for an outer statement plan. TaggedValue describes
Option/Result/payload-enum representation, while operations over those values
may be RuntimeHelper. Interfaces retain explicit dispatch/declaration plans.

The plan MUST distinguish actual type/value, operator, callable/member origin,
construction, declaration/group, statement, control-flow plan, conformance,
file, and erased-alias categories. Every closed mapping input variant MUST
select a plan and every output category MUST verify one. No blanket defaults,
text scanning, string IDs, or global recursive strategy classifiers are allowed.

Required examples:

- Boolean Not is direct unary Not even around a runtime operand. And/Or are
  direct binary roots only when RHS prerequisites are empty; left prerequisites
  alone do not change this. Otherwise check result-local plus conditional plan.
- Wrapping arithmetic, bitwise/float operators and concatenation are direct.
- General equality owns SemanticEqual, including its negated form; enum
  equality owns native equality operators.
- Ordering distinguishes numeric operators, scalar-string runtime comparison,
  and runtime scalar accessors. Conversion/inspection/transformation cases
  select their exact known/runtime/native/fallible root individually.
- OptionUnwrapOr owns a conditional with exact option test/value helpers;
  the fallback is an opaque operand. IsNone/IsErr own negated helper calls.
- Fallible intrinsic calls and ResultPropagation are separate checked mapping
  invocations. Infallible does not imply native: infallible runtime calls exist.
- Alias erasure is Erased, not a fictional Java declaration.

## Proof boundary

Admission and invocation checks detect different failures and need independent
mutation tests. Table-driven mapping tests MUST use builder-fetched wrappers.
Final AST/linker verification, native Java 21 compilation and behavior tests,
cross-target conformance, and fresh review remain required. These certificates
do not prove arbitrary Core-to-AST functional equivalence.
