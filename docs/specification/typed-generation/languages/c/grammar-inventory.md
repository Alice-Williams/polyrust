# C17 closed grammar inventory

- Status: normative migration target; not a claim that all variants are implemented
- Owners: M34A-11-01 through M34A-11-04

This is the exhaustive initial target grammar. A new variant changes this
contract and needs constructor, verifier, renderer and native-oracle coverage.
Names below identify semantic variants, not mandatory Rust type spellings.
Lists are ordinary unbounded containers; target resource admission is separate.
There is no `Other`, raw source, unchecked token, arbitrary macro or string opcode.

## Types and declarators

| Category | Closed variants and payloads |
| --- | --- |
| Scalar | Bool, PlainChar, Int, I8, U8, I16, U16, I32, U32, I64, U64, Size, F64 |
| Object kind | Scalar(scalar), Pointer(target), Array(element, nonzero constant bound), Struct(registration), Union(registration), Enum(registration), Typedef(registration) |
| Object qualifier | Unqualified, Const; array qualification belongs to its element |
| Pointer target | Void(qualifier), Object(object type), Function(exact signature) |
| Return | Void, Value(non-array unqualified object) |
| Parameter | Non-array object with top-level const normalized for signature compatibility |
| Function | Return plus ordered parameter-type list; empty means `(void)` |

Definition-local parameter constness is separate from prototype compatibility.
No function or void is an object. Arrays require complete elements when used;
by-value incomplete types and cycles fail verification. A tag and its typedef
are distinct identities even when their spellings coincide. Qualifier loss,
object/function-pointer conversion and implicit prototype adjustment are not
admitted. Declarator parentheses are derived from this tree, never supplied.

## References, expressions and places

All generated references carry an authoritative registry identity, declaration
kind, origin, owner and structural type. Known library references use closed
catalogue variants. A spelling or matching signature alone authenticates neither.

| Category | Closed variants and payloads |
| --- | --- |
| Literal | Bool(bool), signed/unsigned exact-width integer(value), CharByte(u8), F64Bits(u64), ByteArray(bytes), NullPointer(exact pointer type) |
| Value | Literal, Read(place), FunctionAddress(function reference), Call(callable, ordered arguments), Unary(operator, operand), Binary(operator, left, right), Conditional(condition, then, else), Convert(conversion, operand), SizeOf(complete object type), AlignOf(complete object type), AddressOf(place) |
| Callable | Direct(function reference), Indirect(function-pointer expression with exact prototype) |
| Place | Local(local reference), Parameter(parameter reference), Global(object reference), Member(base place, member reference), Dereference(pointer), Index(base, index) |
| Unary operator | LogicalNot, BitNot, Negate |
| Binary operator | Add, Subtract, Multiply, Divide, Remainder, ShiftLeft, ShiftRight, BitAnd, BitOr, BitXor, Equal, NotEqual, Less, LessEqual, Greater, GreaterEqual, LogicalAnd, LogicalOr |
| Conversion | Numeric(exact destination scalar), AddConst(exact destination pointer), AdapterErase(adapter registration), AdapterRestore(same adapter registration) |

Expressions distinguish void call effects from values; void cannot be an
operand, initializer or argument. Operator signatures use the actual C integer
promotions/usual arithmetic conversions, with explicit conversion nodes when
the portable result differs. Boolean conditions are exact Bool, not arbitrary
truthy pointers or integers. Comparison excludes interface/owned identity.
Null is admitted only for lifecycle/foreign validation, not as an owned value.
No address of a register object or bitfield exists in this subset.

ByteArray is immutable storage with a length, not a NUL-terminated string
assumption. F64Bits lowers to typed bit-transfer storage/operations before the
renderer; it is not permission to invent a target expression during printing.
Arithmetic, numeric conversions, pointer formation, dereferences and indexing
must carry verifier-derived range, extent, initialized-state and provenance
facts. AST inputs cannot manufacture those facts. An adapter registration owns
the exact erased/restored record, interface, witness and function table.

## Initializers, statements and control flow

| Category | Closed variants and payloads |
| --- | --- |
| Initializer | Expression(value), Zero(object type), Array(ordered complete element initializers), Struct(exact registered member initializers), Union(one registered member, initializer) |
| Statement | Empty, Block(ordered statements), Declare(local declaration), Assign(place, value), Evaluate(void/effect expression), If(condition, then block, else block), BoundedLoop(loop registration, body), Switch(value, arms, default block), Break(enclosing loop/switch identity), Continue(enclosing loop identity), Return(optional value), CleanupJump(exit identity), Label(exit identity, statement) |
| Case constant | Exact integer or registered payload-free enumerator, converted to the switch's promoted type before duplicate checking |
| Switch arm | Nonempty list of case constants plus a block; implicit fallthrough is prohibited |
| Loop registration | Counter/bound/step identities with checked initialization, bound, progress and overflow obligations |
| Cleanup exit | Registered forward-only cleanup block with an exact incoming ownership state and destination return/outer exit |

Array/aggregate initializers must cover the required shape exactly; Zero is
type checked, including lifecycle-empty versus inhabited values. Union reads
require the selected member to match the dominating initialization/tag fact.
Calls and other effectful children are sequenced once left-to-right in lowering;
LogicalAnd/LogicalOr and Conditional retain conditional evaluation.
Assignment requires a mutable initialized/initializable place and cannot act
as an owning clone. Assignment expressions, comma expressions and increments
are excluded. CleanupJump is not unrestricted goto. Labels cannot precede a
bare declaration, bypass initialization or skip destruction.

## Declarations, files and directives

| Category | Closed variants and payloads |
| --- | --- |
| Declaration | ForwardTag(struct/union identity), Typedef(identity, aliased type), Aggregate(struct/union identity, ordered members), Enum(identity, nonempty ordered constants), FunctionPrototype(function identity), ObjectDeclaration(object identity) |
| Definition | Function(identity, exact parameter bindings, body), Object(identity, checked initializer) |
| Linkage | External, Internal, None; only legal declaration/context combinations |
| Storage | Automatic, Static, Extern; reject incompatible linkage, file role or initializer combinations |
| File role | GeneratedPublicHeader, GeneratedSource, RuntimePublicHeader, RuntimeSource, PrivateHeader, TestSource, NegativeTestSource |
| File item | Declaration, Definition, StaticAssert(checked constant expression, escaped diagnostic), IncludeGuard(derived file identity), ResolvedInclude(known header or registered local header) |

No anonymous aggregate, tentative public object definition, inline/restrict/
thread-local declaration, executable preprocessor macro or arbitrary directive
is initially admitted. Enumerators fit C int and share the ordinary namespace.
Include/guard items are linker-owned, not attached by portable lowering.
External symbols are unique package-wide; internal symbols cannot appear in
public API references. Test roles cannot supply production definitions.
Deliberate compiler-negative tests use a closed isolated wrong-type fixture
outside production certificates; negative role alone authorizes no invalid AST.

## Proof inventory

Stage 04 must keep a test-owned inventory of each category/variant above and
its positive constructor, rejected mutation and native compiler case. Tests
compare that inventory with exhaustive Rust enum matches; a new variant cannot
silently receive renderer-only coverage. Contextual obligations (namespace,
completeness, ownership, arithmetic, linkage and resource limits) have separate
positive/negative matrices. Until those gates exist, this document specifies
work remaining rather than evidencing a certified C backend.
