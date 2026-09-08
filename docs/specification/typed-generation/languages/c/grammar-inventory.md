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
| Object kind | Scalar(scalar), KnownObject(closed catalogue identity), Pointer(target), Array(element, nonzero constant bound), Struct(registration), Union(registration), Enum(registration), Typedef(registration) |
| Known object identity | File (opaque library object; borrowed pointers only), MaxAlign (complete allocator-alignment carrier) |
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

Typedef targets are registered object types, never bare functions. Expand the
actual registered alias chain before checking parameter/return category and
effective top-level constness. Array aliases still require explicit pointer
adjustment for parameters and cannot be returned. Const hidden by aliases
cannot be stripped from a return or from pointer targets. Prototype-only
top-level normalization preserves pointee/element constness. Alias cycles are
rejected; nominal pointer recursion is a separate completeness obligation.

## References, expressions and places

All generated references carry an authoritative registry identity, declaration
kind, origin, owner and structural type. Known library references use closed
catalogue variants. A spelling or matching signature alone authenticates neither.
Functions and callable members retain a private callable-contract identity in
addition to the C prototype. Generated contracts identify the body whose
ownership/aliasing/failure/evaluation summary must be derived and checked;
known contracts come only from the authoritative closed library catalogue.
An indirect-call proof authenticates the actual function/address/member
provenance against that contract. A same-prototype different function or an
unbound function-pointer field cannot substitute. Contract registration is not
a verified effect summary and cannot manufacture proof transitions.

| Category | Closed variants and payloads |
| --- | --- |
| Literal | Bool(bool), signed/unsigned exact-width integer(value), CharByte(u8), NullPointer(exact pointer type) |
| Value | Literal, Read(place), KnownConstant(closed catalogue entry), Enumerator(registered constant), FunctionAddress(function reference), Call(nonvoid callable, ordered arguments), Unary(operator, operand), Binary(operator, left, right), PointerTest(closed test), Conditional(condition, then, else), Convert(conversion, operand), SizeOf(complete object type), AlignOf(complete object type), AddressOf(place) |
| Effect | Call(void callable, ordered arguments); exact direct/indirect signature, never a value |
| Callable | Direct(function reference with sealed contract identity), Indirect(call-free function-pointer expression, exact prototype and authenticated callable contract) |
| Place | Local(local reference), Parameter(parameter reference), Global(object reference), Member(base place, member reference), Dereference(pointer), Index(base, index) |
| Unary operator | LogicalNot, BitNot, Negate |
| Binary operator | Add, Subtract, Multiply, Divide, Remainder, ShiftLeft, ShiftRight, BitAnd, BitOr, BitXor, Equal, NotEqual, Less, LessEqual, Greater, GreaterEqual, LogicalAnd, LogicalOr |
| Internal pointer test | IsNull(pointer value), IsNonNull(pointer value), SameSlot(left owning-slot pointer, right owning-slot pointer of the exact same type) |
| Conversion | Numeric(exact destination scalar), AddConst(exact destination pointer), ObjectToVoid(exact destination qualified void pointer), AllocationRestore(allocation registration, exact object pointer), AdapterErase(adapter registration), AdapterRestore(same adapter registration) |

Expressions distinguish void call effects from values; void cannot be an
operand, initializer or argument. Operator signatures use the actual C integer
promotions/usual arithmetic conversions, with explicit conversion nodes when
the portable result differs. Boolean conditions are exact Bool, not arbitrary
truthy pointers or integers. Ordinary Binary comparisons are arithmetic only
and exclude pointer/interface/owned identity. Internal PointerTest is separate:
IsNull/IsNonNull accept an authenticated object, void or function pointer and
compare against null of its exact type. SameSlot accepts exact same-type
owning-slot addresses (T**), never two live T* values; it implements the required
self-move guard. Tests produce actual C int, followed by explicit Numeric(Bool)
before use as a condition. These nodes are internal boundary/allocation/lifecycle
operations, not portable Equality operations. Mapping/body certificates check
their actual null-validation or move-guard context; an operation label is not
proof. Pointer ordering and general pointer-versus-pointer identity remain
excluded. Null literals may initialize empty slots without becoming owned values.
Bool renders as native _Bool, not a type macro. Bool literals render an
explicit _Bool conversion of 0/1. C logical/comparison operators and known
integer predicates have their actual int result; an explicit Numeric(Bool)
node converts them before an exact-Bool condition or portable Bool result.
CharByte means an unsigned storage byte with actual U8 type, not a C character
constant. Typed integer/byte literals use explicit exact-type conversions or
suffixes so their C type cannot depend on magnitude; signed minima never spell
an out-of-range signed intermediate. These are fixed literal spellings, not
permission to discover helpers/includes or lower operations while rendering.
KnownConstant entries retain header, actual type and constant-expression
eligibility (including platform properties such as CHAR_BIT/DBL_MANT_DIG).
Enumerator values retain their ordinary-namespace registration and actual C
int type; portable fixed-width tags require explicit conversion. Neither
category can be forged as a literal or treated as an object place.
Null is admitted only for lifecycle/foreign validation, not as an owned value.
No address of a register object or bitfield exists in this subset.

ByteArray and F64Bits are mapping inputs, not final C literal variants.
ByteArray lowers to immutable nonempty array storage/element initializers and
an explicit AddressOf(Index(array place, zero)); empty bytes take the no-buffer
path. Read(array place) is rejected: no implicit value-level array decay is
admitted. Address formation retains the array's nonzero bound and provenance.
F64Bits lowers to U64 storage, a double temporary and typed memcpy operations,
leaving Read(double temporary) as the value. All those declarations/effects
exist before certification; rendering never performs semantic expansion.
Byte storage has an explicit length, not a NUL-terminated string assumption.
Arithmetic, numeric conversions, pointer formation, dereferences and indexing
must carry verifier-derived range, extent, initialized-state and provenance
facts. AST inputs cannot manufacture those facts. An adapter registration owns
the exact erased/restored record, interface, witness and function table.
ObjectToVoid preserves constness, extent and provenance for object pointers;
it is not a function-pointer conversion. AllocationRestore authenticates the
allocator result and required object alignment/extent, retaining its exact
Uninitialized or Prefix(n) state rather than inventing a live value. A typed
place may then be used for initialization writes. Reads require dominating
initialized-member/prefix facts; a complete owner becomes Live only after all
required initialization commits. Restore cannot authenticate an arbitrary
void pointer. AddConst changes
only the immediate pointee qualification and cannot admit T** to const T**.
Exact call/initializer typing does not hide these conversions implicitly.

## Initializers, statements and control flow

| Category | Closed variants and payloads |
| --- | --- |
| Initializer | Expression(value), Zero(object type), Array(ordered complete element initializers), Struct(exact registered member initializers), Union(one registered member, initializer) |
| Local declaration | Registered local reference plus optional exact initializer; Automatic storage only |
| Statement | Empty, Block(scope registration, ordered statements), Declare(local declaration), Assign(place, value), Evaluate(effect), Discard(value), If(condition, then block, else block), BoundedLoop(loop registration, counted progress, explicit condition, body), Switch(switch registration, value, arms, default block), Break(innermost loop/switch identity), Continue(innermost loop identity), Return(optional value), CleanupJump(exit identity), Label(exit identity, statement) |
| Case constant | Exact integer or registered payload-free enumerator, converted to the switch's promoted type before duplicate checking |
| Switch arm | Nonempty list of case constants plus a block; implicit fallthrough is prohibited |
| Loop registration | Existing function/scope-owned identity, bound to exactly one BoundedLoop occurrence |
| Counted progress | Exact counter and bound local references plus closed Step::One; actual initialization/condition/update AST is independently checked, not manufactured by this metadata |
| Cleanup exit | Registered forward-only cleanup block with an exact incoming ownership state and destination return/outer exit |

The exact initial counted-while form, complete AST payload and ownership of
initialization/condition/update evidence are normative in
[counted loops](counted-loops.md). There is no implicit for-loop expansion or
renderer-supplied step, direction, bound test or Continue fixup.

Array/aggregate initializers must cover the required shape exactly; Zero is
type checked, including lifecycle-empty versus inhabited values. Union reads
require the selected member to match the dominating initialization/tag fact.
Calls and other effectful children are sequenced once left-to-right in lowering;
LogicalAnd/LogicalOr and Conditional retain conditional evaluation.
Every call is conservatively sequencing-required, including known calls; there
is no caller-supplied purity flag. A call may occur only as a full-expression
root in a standalone automatic local's Expression initializer, an assignment
to a direct Local place, Return, Discard or Evaluate, with call-free operands.
Calls are forbidden inside aggregate/array initializers and inside indexed,
member or dereferenced assignment places/RHS pairs; materialize their result
in a local first. Other expressions, places and conditions are call-free. Lowering
materializes child calls into ordered statements; short-circuit/conditional
prefixes stay inside the selected branch rather than being eagerly hoisted.
Stage 02D rejects nested/sibling argument calls or any hidden call in an
operand/condition. Mapping certificates separately authenticate the original
child/prefix order; target validity alone cannot know source evaluation order.
Assignment requires a mutable initialized/initializable place and cannot act
as an owning clone. Assignment expressions, comma expressions and increments
are excluded. CleanupJump is not unrestricted goto. Labels cannot precede a
bare declaration, bypass initialization or skip destruction.
Each loop/switch registration binds exactly one structural statement occurrence
in its registered function/scope. Break must name the actual innermost enclosing
loop or switch; naming an outer construct is rejected because C break is not
labelled. Continue must name the innermost loop, ignoring intervening switches.
The verifier derives these stacks from the AST and rejects reused, crossed or
unbound control registrations; no caller-supplied enclosing-target fact suffices.
The Local declaration's reference supplies its exact type and lexical owner.
Every structural block binds a registered scope exactly once. A function body
binds its own root scope (no parent); each nested block binds a child whose
registered parent is the actual containing block scope and whose function
owner matches. Every registered scope has one corresponding block occurrence.
Declarations belong to their exact scope; references may read that scope or a
legal descendant only after declaration/initialization dominance. Sibling
scope identity cannot be inferred from statement order or matching spellings.
Swapped siblings, duplicated/absent scope occurrences and wrong parents fail
verification; legal nested reads and independently named shadow bindings remain
possible under namespace/name allocation rules.
No initializer means Uninitialized, never an implicit zero value. Const locals
require an initializer. A typed initializer establishes precisely its covered
object/member/prefix state; Zero for an owner establishes Empty, not Live.
Local static storage is excluded; Static/Extern apply only to their legal
file-level object/function contexts. There is no hidden local static singleton.

## Declarations, files and directives

| Category | Closed variants and payloads |
| --- | --- |
| Declaration | ForwardTag(struct/union identity), Typedef(identity, aliased object type), Aggregate(struct/union identity, nonempty ordered members), Enum(identity, nonempty ordered constants), FunctionPrototype(function identity), ObjectDeclaration(object identity) |
| Definition | Function(identity, exact parameter bindings, body), Object(identity, checked initializer) |
| Linkage | External, Internal, None; only legal declaration/context combinations |
| Storage | Automatic, Static, Extern; reject incompatible linkage, file role or initializer combinations |
| File role | GeneratedPublicHeader, GeneratedSource, RuntimePublicHeader, RuntimeSource, PrivateHeader, TestSource |
| File item | Declaration, Definition, Comment(normalized non-executable documentation), StaticAssert(checked constant expression, escaped diagnostic), IncludeGuard(derived file identity), ResolvedInclude(known header or registered local header) |

No anonymous aggregate, tentative public object definition, inline/restrict/
thread-local declaration, executable preprocessor macro or arbitrary directive
is initially admitted. Enumerators fit C int and share the ordinary namespace.
Include/guard items are linker-owned, not attached by portable lowering.
An aggregate definition has at least one registered member. Forward tags may
remain incomplete; they are not empty definitions. Portable empty records use
nonempty private bookkeeping layouts. Constructor, mutation and strict native
controls reject empty struct/union definitions while accepting forward tags.
External symbols are unique package-wide; internal symbols cannot appear in
public API references. Test roles cannot supply production definitions.
Discard renders a void conversion without allowing void as an ordinary value.
Comment normalization follows platform-and-proof.md; user text never owns
delimiters. KnownObject identities are distinct from generated Typedef
registrations. The closed identity fixes FILE versus max_align_t;
catalogue-owned metadata supplies the header, completeness and measured ABI
facts, never caller flags. FILE is admitted only behind a borrowed pointer,
never by value, as an array element, in SizeOf/AlignOf or as constructed
storage. MaxAlign is a complete known object with the measured model's layout
and alignment. A generated type with the same name cannot substitute for
either catalogue identity. Stage 02B owns the identity-bearing grammar and
actual constant types; stage 03 completes authoritative catalogue metadata,
callable signatures/effects and dependency resolution.
The initial public ABI supports C consumers, not C++ linkage
wrappers; no unmodelled extern-language directive is emitted.
Deliberate compiler-negative tests use a closed repository-native-oracle fixture,
outside plugin packages, manifests and certificates. There is no invalid-source
file role in the production C AST. The test-only fixture renderer cannot be
called by a backend adapter or produce an OutputManifest.

## Proof inventory

Stage 04 must keep a test-owned inventory of each category/variant above and
its positive constructor, rejected mutation and native compiler case. Tests
compare that inventory with exhaustive Rust enum matches; a new variant cannot
silently receive renderer-only coverage. Contextual obligations (namespace,
completeness, ownership, arithmetic, linkage and resource limits) have separate
positive/negative matrices. Until those gates exist, this document specifies
work remaining rather than evidencing a certified C backend.
