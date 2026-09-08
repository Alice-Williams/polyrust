# C17 capability strategy inventory

- Status: normative migration target, not implemented Supports claims
- Owner: M34A-11-07; shared catalogue: crates/build/src/capabilities/mod.rs

Exactly one capabilities/<snake_case>.rs module owns each row. Each mapping
has its own closed C<Capability>Input and sealed Plan enums; singleton inputs
may be private-field structs. No string operation ID or erased all-intrinsics
input substitutes for these types. Variant names below are the C contract,
not a requirement to preserve Java's type names.

## Common exact input and proof fields

Every input carries its checked source identity and exact CoreIR type, plus
already-lowered typed child plans. A type input carries the complete concrete
type arguments; a value input carries its exact literal or ordered children.
Unary/binary/ternary operations have respectively one/two/three children with
the source operation's exact argument types. Function/member reads and calls
carry registered owner, declaration, signature and binding identity. No
caller-supplied result type overrides the type derived from those identities.

Each plan binds its input variant, monomorphization, exact operand/result
types, ordered child slots, mapping-owned AST root, helper references,
declaration/call signatures and ownership/failure obligations. The checked
wrapper selects it before invoking the stored handler and verifies it against
the returned AST. Wrong branches, ignored operands, changed signatures,
forged ownership, wrong helper or substituted operators cannot authenticate.
Per-row mutation tests alter each of these fields independently.

Outputs: Type = CObjectType plus registered nominal declarations;
Value = typed expression with ordered prefix, cleanup and effect information;
Decl = complete registered declaration/definition bundle; Control = scoped
statement/control-flow plan; Package = typed file/declaration inventory;
Test = typed invocation/assertion/completion inventory. Each capability owns
its concrete output enum; these labels are categories, not a shared erased
output type.

All rows inherit inferred requirements of their concrete child types/bodies.
The last column names additional semantic constraints/prerequisites, not
manual includes or a second runtime support registry. Allocation (A) means
new owned storage; portable failure (P) means the established computational
error channel. All public callable wrappers additionally have ABI transport
failure, including outcome allocation, per [callable ABI](callable-abi.md).
Pure scalar AST operations do not themselves allocate.

## Exhaustive catalogue

| Capability | Closed inputs | C output and selected strategy | Additional obligations |
| --- | --- | --- | --- |
| Functions | Declaration, ParameterRead, Return, Call | Decl/Value/Control; exact outcome ABI, sequenced arguments and cleanup exits | Registered signature; ResultPropagation for fallible calls |
| Records | Type, Declaration, Construction, Field | Type/Decl/Value; opaque nominal handle, ordered field factory and immutable projection | A construction/clone; exact field owner/type; no layout heuristic |
| BoolValues | Type, Value(bool) | Type/Value; Bool literal | None |
| I32Values | Type, Value(i32) | Type/Value; exact int32_t literal | Exact minimum representation |
| I64Values | Type, Value(i64) | Type/Value; exact int64_t literal | Exact minimum representation |
| F64Values | Type, Value(u64 bits) | Type/Value; binary64 bit-transfer helper, never decimal approximation | Verified memcpy bit representation |
| TextValues | Type, Value(UTF-8 bytes) | Type/Value; validated immutable UTF-8 owner | A; length, scalar validity and embedded zero |
| BooleanLogic | Not, And, Or | Value; Bool operators, RHS prefix remains conditional | BoolValues; no eager RHS |
| Equality | Equal, NotEqual | Value; recursive IEEE comparator except top-level enums | Recursively equatable types only; no interface/address equality; Enums owns enum equality |
| Ordering | Less, LessEqual, Greater, GreaterEqual | Value; checked scalar or Unicode-scalar lexical ordering | Exact admitted operand category; no pointer comparison |
| CheckedIntegerArithmetic | Neg, Add, Subtract, Multiply, Divide, Remainder | Value; signed range guards before native operation | P; zero and minimum/-1; ResultPropagation |
| WrappingIntegerArithmetic | Neg, Add, Subtract, Multiply | Value; unsigned-width arithmetic and proved signed reconstruction | I32/I64 width; no implementation-defined overflow shortcut |
| FloatingPointArithmetic | Neg, Add, Subtract, Multiply, Divide, Remainder | Value; binary64 operators, fmod for remainder | F64Values; pinned FP environment, typed Math link requirement |
| StringConcatenation | Concat(left, right) | Value; length-checked immutable UTF-8 concatenation | A; TextValues |
| CharValues | Type, Value(char) | Type/Value; validated uint32_t scalar | Exclude surrogate and out-of-range values |
| BytesValues | Type, Value(bytes) | Type/Value; immutable byte owner and explicit length | A; no strlen |
| ListValues | Type(element), Value(ordered elements) | Type/Value; monomorphized immutable owner | A; initialized prefix, element lifecycle |
| OptionValues | Type(element), None, Some(value) | Type/Value; opaque tag/payload, None is a live value | A; exact element lifecycle and tag dominance |
| ResultValues | Type(ok,error), Ok(value), Err(value) | Type/Value; ordinary opaque tagged value | A; distinct from computational outcome |
| IntegerBitwise | Not, And, Or, Xor | Value; exact unsigned-width bit operations/reconstruction | I32/I64; promotion checked |
| CheckedIntegerShifts | Left, Right | Value; checked count and portable signed-shift algorithm | P; no invalid shift evaluated; ResultPropagation |
| FloatingPointInspection | Truncate, IsNan, IsNegativeZero, Absolute | Value; trunc/isnan or raw-bit masks as appropriate | F64Values; exact FloatAbs payload audit |
| StringInspection | ScalarLength, Utf16Length, IsEmpty, IndexOfLiteral, Contains, StartsWith, EndsWith | Value; length-aware UTF-8/scalar scans and exact index units | TextValues; P where portable size conversion is checked |
| StringTransformation | StripPrefix, TruncateUtf8Bytes, TrimStart, TrimEnd, SliceScalars, ReplaceAll, ReplaceMany | Value; scalar-boundary typed loops and fresh immutable output | A; exact ordered replacement/empty-needle behavior |
| BytesOperations | Length, IsEmpty, Concat, ReplaceAll | Value; byte-indexed bounded loops | BytesValues; A for new owner |
| ListOperations | Length, IsEmpty, GetChecked, Append, Concat, Contains, IndexOf | Value; monomorphic loops, clone on owned extraction | A for owned output; P bounds; searches require recursive equality |
| OptionOperations | IsSome, IsNone, UnwrapOr | Value; tag observations, correctly sequenced fallback selection | OptionValues; clone/transfer selected owned value |
| ResultOperations | IsOk, IsErr | Value; ordinary Result tag observations | ResultValues; no computation-error conflation |
| IntegerConversions | WidenI32ToI64, NarrowI64ToI32Checked | Value; exact widening or pre-cast range guard | P narrowing; ResultPropagation |
| Utf8Conversions | Encode, DecodeChecked | Value; immutable bytes/text factory and strict scalar decoder | A; P invalid UTF-8; BytesValues/TextValues |
| Modules | Declaration(inventory) | Package; registered public/private files and exact declaration inventory | Actual registrations in both directions; not structural orchestration |
| Constants | Declaration, Reference | Decl/Value; allocator-parameterized fresh-value getter | A owning results; checked dependency DAG; immutable backing only |
| TypeAliases | Declaration(name, exact target), TypeReference | Decl/Type; registered typedef, transparent semantic target | Alias cycle check and nominal ownership retained |
| Enums | Type, Declaration, Variant, Equality, Branch, PayloadDeclaration, PayloadConstruction, PayloadPattern, PayloadBindingRead, PayloadBranch | Type/Decl/Value/Control; fixed-width validated tags; opaque legacy payload plan | Owns all top-level enum equality/matching, including aliases and C legacy compatibility; A legacy payload; exact tag/member dominance |
| Interfaces | Type, UninhabitedType, Declaration, ImplementationBundle, SelfValue, Coerce, ConcreteCall, InterfaceCall | Type/Decl/Value; exact flat table/context adapter and lifecycle bundle | A clone/coercion; exact witness; empty implementation set valid |
| PortableTests | FunctionInvocation, MethodInvocation, Case, Harness | Test; outcome/status assertions and exact completion inventory | NaN-class expectation comparator; string error codes; no vacuous main |
| LocalBindings | Bind, Read | Control/Value; scoped registration, immutable read or explicit owned clone | Exact binding identity and lifetime |
| Conditionals | Value(condition, then, else) | Control/Value; branch-local prefixes and joined output ownership | BoolValues; both reachable exits satisfy result type/state |
| Loops | ForEach, BindingRead | Control/Value; bounded iteration and scoped element binding | ListValues; count/index bounds, loop-carried ownership joins |
| PatternMatching | Pattern, Match, BindingRead | Control/Value; exhaustive non-enum tag/Bool switch, dominated projections | Closed patterns Wildcard/Bool/None/Some/Ok/Err; no top-level enum matches; no fallthrough |
| ResultPropagation | Propagate(call outcome) | Control/Value; separate transport and portable-error exits | Cleanup all live temporaries, retain ordinary Result values |
| UnitValues | Type, Value | Type/Value; uint8_t zero | Foreign input domain check |

C runtime implementation subroutines (allocation, clone/drop, UTF-8 scanning)
are structural dependencies of these mappings, not new public capabilities.
A mapping cannot require its own support by recursively consulting an empty
slot: concrete type/lifecycle specializations are registered once and linked
as a dependency graph. Unsupported recursive layout/specialization shapes
need explicit diagnostics and specification, never an infinite expansion.

Typed payload-free enum equality and exhaustive branching belong to Enums
alone, as required by the shared catalogue; they do not infer Equality or
PatternMatching. C also deliberately assigns its legacy payload-enum equality,
matching and payload binding operations to Enums. That compatibility choice is
C-specific, not a claim that the shared typed frontend exposes tagged unions.
Transparent aliases preserve this dispatch category. Recursive comparison of
enum fields within another aggregate uses the registered structural comparator
dependency; it does not invoke a second unregistered capability mapping.
PortableTests owns its separate expectation comparator. Tests must prove an
Enums-only typed program needs neither Equality nor PatternMatching, and that
legacy enum/wildcard/binding paths select exactly one authenticated owner.

## Required evidence per variant

The test-owned inventory is compared with both the shared 42-marker catalogue
and exhaustive C input/plan matches. Each variant records: positive mapping
invocation, plan mutation rejection, verified AST category, strict native
compile, native behavior, and applicable ownership/fault-injection cases.
A scalar success fixture alone cannot certify all variants of its capability.
Requirements are inferred from actual generic AST usage, not this table's
prose or a feature-name string. Partial registration remains unsupported until
the row's entire closed input family passes.
