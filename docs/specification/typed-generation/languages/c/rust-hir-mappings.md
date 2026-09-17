# Rust HIR to existing C types and nodes

- Status: normative M35 design; mapping implementation and proof pending
- Parent: [C Rust-source lowering](rust-hir-lowering.md)

## Type reuse

The implementation names below are authoritative. Older specification shorthand
`CType`, `CExpr`, `CStmt` and `CFile` MUST NOT become duplicate types.

| Compiler-admitted input | Existing C representation | Required mapping rule |
| --- | --- | --- |
| i32 | CObjectType with CScalarType::I32 | Exact fixed width; not implementation-sized int |
| i64 | CObjectType with CScalarType::I64 and CStdType::I64 | Exact int64_t identity, stdint dependency and conditional LP64 size/alignment obligations |
| bool | CObjectType with CScalarType::Bool | Native _Bool; comparisons may require explicit conversion from C int |
| Nonempty, non-generic no-Drop struct with only i32/i64/bool fields | CStructRef, CMemberRef and CObjectType::structure | Private value layout; distinct nominal owners even for identical fields |
| Shared reference to an admitted local object | CObjectType::pointer, CPointerTarget::Object, CConstness::Const on the pointee | Preserve reference depth, target type and qualifiers; not nullable ownership |
| Function signature | CFunctionType, CParameterType, CReturnType and CReturnValue | Exact prototype; no variadics, unspecified-argument declarations or callable casts |
| Integer/bool literal | CLiteral and CSignedLiteral where applicable | Use checked literal constructors and exact minimum-value spelling |
| Type alias | Compiler-normalized type plus original declaration provenance | No alias-based admission bypass; alias provenance is not pointer/layout proof |

Until the alias-use provenance row is implemented, the intermediate compiler
bridge explicitly rejects alias and associated-type paths in HIR type surfaces.
The initial conservative scan covers the analyzed crate, including signatures,
record fields, annotated locals and constructor paths. An unused alias declaration alone is not a
use. Normalized primitive types must not silently bypass this admission guard.

All types use existing checked constructors. A shared reference may not be
turned into a mutable pointer. Pointer constness is not a lifetime proof.
The first implementation admits only local references with demonstrably
preserved storage and no escaping references or borrowed aggregate fields.
No aggregate ABI/layout equivalence with Rust is promised: generated C has
its own private representation and no cross-language memory sharing.

Packed/custom-aligned representations, unions and address/layout observation
are not implicitly admitted because their fields happen to be scalar.
Additional widths, floats, chars, unit, arrays and unsized types require
separate explicit mappings even if the C AST already represents them.

## Declarations, expressions and control

| HIR/type-check fact | Existing typed C output | Mapping obligation |
| --- | --- | --- |
| Resolved function/declaration | CFunctionRef, CDeclaration, CDefinition | Authenticate declaration origin and signature; allocate protected names through CIdentifier/registry |
| Parameter/local binding | CParameterRef / CLocalRef with CScopeRef | Resolve by compiler identity, including shadowing; spelling never selects a binding |
| Plain initialized let | CLocalDeclaration, CInitializer, CStatementKind::Declare | Evaluate initializer before binding; preserve lexical storage duration |
| Local read or admitted value move | CPlace plus CValueKind::Read | C copy implements the admitted no-Drop value move only; moved-from Rust value is not reused |
| Field access | CPlaceKind::Member with CMemberRef | Compiler field index must resolve to that exact nominal owner's registered member |
| Shared borrow / built-in dereference | CValueKind::AddressOf / CPlaceKind::Dereference | Apply compiler adjustment metadata exactly; reject overloaded Deref and unimplemented coercions |
| Scalar comparison | CBinaryOperator and CValueKind::Binary; CConversion::Numeric when needed | Exact operand types and C promotions; Rust reference comparison is not C pointer comparison |
| Built-in bool negation | CUnaryOperator::LogicalNot plus CConversion::Numeric(Bool) | Preserve one operand evaluation; reject integer bit-not, overloads and implicit adjustments |
| Built-in bool lazy and/or | Initialized private bool local and CStatementKind::If/Assign | Left once; entire right prelude inside the selected child scope; no eager RHS computation |
| Built-in i32/i64 bitwise operations | CUnaryOperator::BitNot and CBinaryOperator::BitAnd/BitOr/BitXor | Equal exact-width operands; C I32 promotion normalized back from Int; ordered calls; see M35-03A-02D |
| Struct initialization | CInitializerKind::Struct | Match the complete ordered member inventory; preserve source evaluation order separately |
| Nested source block | CBlock and CStatementKind::Block | Keep its scope and braces; do not flatten into the enclosing block |
| If/else | CStatementKind::If with two CBlocks | Evaluate condition once; evaluate only the selected branch |
| Tail expression / admitted explicit return | CStatementKind::Return | Produce the function result after required sequencing; no implicit fallthrough |
| Sequencing temporary | Registered CLocalRef and CStatementKind::Declare/Assign | Synthesized evaluation-temporary provenance, exact type and owning scope |

`CExpressions`, `CStatements` and `CDeclarations` remain the constructor
families. Do not make private fields public or construct detached enum payloads
to evade registry authentication.

The first implementation MUST cover nested tail blocks and if/else returns.
General expression-valued blocks, early returns and if-valued initializers are
separate admission shapes: when added, lower them using a destination local
outside the branches, branch-local assignments and preserved inner CBlocks.
Reject until implemented; the renderer must not reconstruct such control flow.

Preserve Rust's evaluation order. A C initializer or argument list is not
an ordering guarantee. For effectful operands, emit sequenced typed statements
in source order before arranging initializer members in declaration order.
The initial side-effect-free subset must prove that restriction explicitly.
Do not eagerly lower a short-circuit RHS or evaluate both conditional branches.

Built-in and overloaded operations have different admission. Authenticate
type-dependent resolved calls and every implicit adjustment. Unknown callable
identity, auto-borrow, pointer coercion, unsizing or overloaded dereference
rejects unless its exact mapping has been registered.

Unused parameters and locals are valid source shapes within an otherwise
admitted function. Lowering must handle them without breaking strict C warnings,
for example with a typed Discard for initialized side-effect-free values;
it must not introduce a raw warning pragma or silently discard effects.

## Capabilities outside the first subset

The [integer bitwise extension](../../rust-integer-bitwise.md) adds a separate
IntegerBitwise executable slot. Its shared private input validates built-in
operation identity, operand/result types and absent adjustments. It does not
enable Boolean eager operators, shifts, arithmetic, casts or other widths.
The separate [eager Boolean extension](../../rust-eager-booleans.md) maps bool
And/Or/Xor through its own executable slot. Its Bool operands promote to Int;
normalize that result back to Bool and retain once-only left-to-right calls.

The [i64 extension](../../rust-i64-values.md) reuses these executable capability
slots and adds no broad numeric escape hatch. A shared checked input interprets
literals, including signed minima, before target mapping. C uses registered
int64_t references and explicitly typed minimum-value spelling. The dependency
traversal retains scalar identities as well as headers; platform assertions do
not count as source uses and cannot request their own layout obligations.
Source arithmetic, other integer widths and integer writes remain unsupported.

### Executable source mapping ownership

The compiler bridge registers these narrow source-capability owners. They are
not aliases for the complete portable capability catalogue:

| Source capability | Session-bound input | C output |
| --- | --- | --- |
| ObjectTypes | compiler Ty | CObjectType plus registered nominal declarations |
| LocalConstants | private compiler-derived LocalConstantInput retaining the item statement/DefId and exact bool/i32/i64 value | Unit output; validate the declaration and erase it without runtime storage |
| ScalarConstants | private evaluated ConstantInput retaining compiler DefId/expression and exact bool/i32/i64 value | Ordinary typed literal CValue, no runtime storage |
| LiteralValues | private checked LiteralInput with typed bool/i32/i64 value and compiler-session lifetime | CValue |
| ResolvedPlaces | resolved path/field/dereference HIR expression and adjustments | CPlace |
| SharedBorrows | immutable built-in borrow HIR expression | CValue |
| ScalarComparisons | resolved scalar-comparison HIR expression | CValue |
| BooleanNegation | checked private NegationInput retaining the bool operand's HIR | CValue |
| ShortCircuitBooleans | checked private LazyBooleanInput and typed And/Or operator | CValue plus scoped Boolean evaluation statements |
| IntegerBitwise | checked private BitwiseInput retaining exact-width operands and closed complement/And/Or/Xor shape | CValue with exact-width result normalization |
| EagerBooleans | checked private EagerBooleanInput retaining Bool operands and closed And/Or/Xor operator | CValue with Int-to-Bool normalization |
| RecordInitializers | complete scalar-field struct HIR initializer | CInitializer |
| LexicalControl | HIR expression, optional parent HIR identity and Return/Effect completion | CBlock |
| EntrySignatures | selected compiler function identity and signature facts | CFunctionType |
| FunctionSignatures | ordinary local or authenticated foreign compiler DefId and scalar-parameter / scalar-or-unit-result signature facts | CFunctionType |
| DirectCalls | resolved ordinary scalar-result call HIR expression | CValue plus scope-owned typed evaluation declarations |
| UnitEffects | private checked UnitInput with closed empty/call/block/conditional operation and HIR scope | Typed effect statements; ordinary void calls and no value temporary |

Each binding has associated input/context/output types and an executable lower
method. A consuming typed-slot builder permits one registration per capability;
Supports obtains the stored mapping, never an independently asserted flag.
Capability owns only source input; Mapping owns target-specific context and
output. No source-capability input may contain a target AST reference. In
particular, lexical control resolves its parent HIR identity through the C
Reader's authenticated scope bindings. An absent parent binding or repeated
current identity is a diagnostic, never an implicit new root or overwrite.
Traversal selects a binding and passes compiler references rather than cloning
HIR into a private source tree. Admission of dynamic HIR shapes remains checked
by its owning capability input constructor and mapping; a Rust type wrapper is not a claim that arbitrary
customer syntax was checked when the generator itself was compiled.

The initial slot refactor may migrate owners incrementally but cannot be marked
complete while operation rules remain duplicated in traversal. See
[M35-01B-02](../../../../plan/tasks/M35-01B-02-source-capabilities.md).

### Later capabilities

The same-crate call extension is specified separately in
[direct calls](rust-hir-direct-calls.md). Its effect and native-stack evidence
must pass before expanding the render-ready profile. Its two executable
bindings established ten required slots; [Boolean negation](../../rust-boolean-negation.md)
adds an eleventh executable slot; [short-circuit Boolean expressions](../../rust-short-circuit-booleans.md)
add the twelfth; IntegerBitwise adds the thirteenth and EagerBooleans the fourteenth. ScalarConstants adds the fifteenth; LocalConstants adds the sixteenth. Missing and duplicate
registration controls cover each slot independently. Typed Boolean logical-not
and exact-width integer bit-not are admitted by the closed scalar-call evidence
and shared-package profiles. Arithmetic negation remains outside this profile.
Every operand is still traversed and checked.

Short-circuit lowering uses initialized bool locals and two explicit child
blocks. Only direct local-bool assignments enter the closed shared/effect
profiles; parameter, member, pointer and other scalar writes remain rejected.
Registry scope, definite initialization, ownership/storage and resource checks
remain mandatory. Rendering consumes those statements without adding control
flow or custom runtime dependencies.

These are extension obligations, not current support claims:

- Further calls and generics: exact resolved callable identities, monomorphized
  signatures, explicit argument sequencing and recursion/resource policy.
- Integer operations: map the selected Rust overflow/panic/wrapping semantics;
  C signed overflow is never an acceptable implementation.
- Mutable references/interior mutability: prove target storage, aliasing and
  mutation behavior; do not reduce admission to a const-qualifier check.
- Heap owners and Drop: compiler-derived ownership/drop facts with authenticated
  correspondence to structured source exits; exact panic/allocation policy.
- Loops: HIR may contain desugared loop/match forms. Match an admitted semantic
  pattern and use supported typed loop nodes, or extend the grammar with proof.
  An arbitrary Rust loop is not an existing C bounded-loop certificate.
- Payload-free enums: use the decided nominal enum mapping with validated
  tags. Rust enums with payloads remain a separate tagged-value capability.
- Traits/interfaces: use exact trait/implementation/method identities,
  existing CInterfaceWitnessRef/CInterfaceTableRef/CInterfaceAdapterRef and
  typed CCallable/CCall contracts. Static calls and dynamic table dispatch
  are separate mappings within interface support, not inheritance.
- Documentation: apply the [documentation contract](rust-hir-documentation-and-proof.md).

Composition and flat interface tables remain the design. No inheritance,
layout-prefix downcasts, container-of tricks or string-selected methods.
Missing C AST support requires an explicit typed extension and its tests;
neither a raw-code escape nor the experimental miniature model is a fallback.

## Compiler-evaluated constant reads

[M35-03A-02F-01](../../../../plan/tasks/M35-03A-02F-01-constant-reads.md)
implements the [scalar constant contract](../../rust-scalar-constants.md).
Resolve nongeneric module/inherent constant paths with rustc and map evaluated
bool/i32/i64 values to existing CLiteral/CSignedLiteral nodes. Do not infer a
value from spelling or emit copied runtime storage. Public constant API names,
trait/generic constants and constant borrows must reject until their explicit
mappings exist. Initializer arithmetic evaluated
by rustc does not enable runtime arithmetic.

## Block-local constant declarations

[M35-03A-02F-02A](../../../../plan/tasks/M35-03A-02F-02A-local-constants.md)
extends lexical item admission under the shared
[local constant contract](../../rust-local-constants.md). LocalConstants uses
the same compiler evaluator as ScalarConstants and validates unused declarations.
Its executable C mapping returns unit: no local, static object, prelude, helper
or import is registered. Constant reads remain exact typed literals resolved by
compiler DefId, including forward references and nested same-name definitions.
Other local item kinds reject; public constant exports remain a separate step.
