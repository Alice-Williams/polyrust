# C17 deterministic callable and lifecycle ABI

- Status: normative design target; implementation evidence belongs to M34A-11-02–06
- Refines [ownership and safety](ownership-and-abi.md)

## Representations

No representation is selected heuristically from whether a record happens to
contain owned fields. Every record, String, Bytes, List, Option, Result and
interface is a distinct opaque owning handle. Legacy payload enums are also
opaque. A public handle typedef names its incomplete struct; inputs use a
pointer to const, owned output slots use a pointer to pointer to mutable
incomplete struct. The implementation alone defines layouts.

Unit is uint8_t with the single valid value zero. Bool is _Bool; I32/I64 use
int32_t/int64_t; F64 is double; Char is uint32_t restricted to Unicode scalars.
Each payload-free enum has a distinct typedef to uint32_t and a closed set of
zero-based tags allocated by canonical variant identity order. Its nominal
identity is retained in Rust registrations despite C typedef compatibility.
Native C enum storage is never the portable ABI representation. Named C enum
declarations may supply integer constant names; generated enum constants fit
C int. More variants are a generation-capacity error, not silent truncation.
An empty portable enum has the fixed-width typedef but no named native enum
declaration and no valid tag; all foreign tag constructions fail InvalidInput.
Similarly, a zero-field portable record remains an opaque live owner with
private allocator/bookkeeping fields; it never emits an empty C aggregate.
Every foreign enum/Char/Unit input is validated before use. Option/Result and
legacy payload-enum discriminants are private uint32_t tags; active-member
facts dominate every payload read.

## Callable family

Write S(T) for the scalar C representation and H(T) for an opaque handle.
Borrow(T) is S(T) by value for scalars and const H(T)* for owned values.
Every generated portable function has the following exact ordered ABI:

    poly_status function(const poly_allocator *allocator,
                         Borrow(A0) arg0, ...,
                         poly_outcome_R **out);

The parameter list is variadic only in this specification's notation; the C
prototype has an exact finite list and never C varargs. No frontend arity cap
is introduced. Methods add their exact borrowed receiver immediately after
allocator. Direct calls and vtable slots share the same argument/outcome ABI.

poly_outcome_R is an opaque owned computational outcome, distinct from ordinary
portable Result<T,E>. On transport success it contains either the complete R
value or the established portable error code and message. Its observations
expose the branch and borrowed immutable value/error views; extracting an
owned value requires clone. There is no public take operation in this ABI.
The initial ABI allocates an outcome even for scalar returns; allocation-free
specializations are a later explicit ABI/design change, not a hidden heuristic.

The exact factory, projection, tag and view signatures are specified in
[public value ABI](public-value-abi.md); observers are not implementation-chosen
overloads. Public value factories return transport status plus the value's
owning slot, not a computational outcome.

poly_status has uint32_t ABI and closed values Success(0),
AllocationFailure(1), Capacity(2), InvalidInput(3), InvalidState(4). Capacity
covers valid-input size/growth/conversion bounds; it is distinct from memory
allocation returning null. These statuses do not replace portable computation
errors or ordinary Result Err payloads. A transport failure preserves the
initially empty output slot, releases every partial allocation and preserves
borrowed inputs. A rejected nonempty output remains unchanged. An outcome
containing a portable error is transport Success.

The allocator argument may be null to select the default allocator. The output
slot must be a valid writable address and initially null. A non-null output
is InvalidState and is neither overwritten nor dropped. Detectable null
required inputs, malformed tags or invalid views yield InvalidInput before
dereference. Arbitrary forged pointers cannot be validated by C; the foreign
caller must supply valid storage and meet documented lifetimes.

## Validation order and combined failures

Every public portable function/method, constant getter, factory/coercion/clone,
observer/tag accessor and lifecycle operation uses this ordered ladder. Skip
inapplicable phases; stop at the first failure, without performing later
checks, invoking allocation callbacks or modifying any input/output storage.
Within a phase visit formal parameters in signature order, then fields in
declaration order and elements by increasing index.

| Order | Check and failure |
| --- | --- |
| 1 | Top-level required pointer arguments and writable output/slot addresses are non-null: InvalidInput. Allocator may be null; raw array data may be null exactly when count is zero. This pass does not dereference pointers to inspect nested values. |
| 2 | A supplied custom allocator has both non-null callbacks: InvalidInput. Its descriptor storage is a caller precondition; null context is legal. |
| 3 | Owning output slots are empty: InvalidState. Move additionally rejects identical slot addresses and an empty/moved source: InvalidState. Empty drop succeeds here. |
| 4 | Checked span/count/size arithmetic required to traverse inputs: Capacity on overflow, before pointer formation or element access. |
| 5 | Detectable input domains, private tags, UTF-8 and nested required owned values: InvalidInput. Validate a tag before its payload; nested array/field validation follows source order. Valid storage, alignment, extent and borrow lifetime remain caller preconditions, not inspectable guarantees. |
| 6 | Required accessor branch is active: InvalidState; requested observer index is in range: InvalidInput. No payload/index read occurs before its check. |
| 7 | Remaining operation/result/growth capacity calculations: Capacity. |
| 8 | Execute/allocate in specified evaluation order: AllocationFailure on null allocation, with complete rollback; commit outputs only on Success. |

Every nested traversal establishes its own checked extent before access; phase
4 is not permission to pre-read fields behind unchecked tags. These phases
describe public-boundary validation, not portable checked-operation errors:
portable errors still travel inside a successful computational outcome.
Generated-call contracts and mapping verification authenticate this ordering.

Boundary domain validation covers caller-supplied scalar/raw-array inputs and
the immediate owner/tag observations required by the operation. It does not
recursively rescan an already API-constructed private owner graph before every
call: those owners retain their construction invariants. Iterative execution
still checks each selected tag before payload access; a detected corrupted
private tag returns InvalidInput with rollback, not an unchecked read. This is
not a promise to discover all foreign graph corruption before any work allocation
or to validate forged pointers. Work-storage failures follow actual ordered
execution, as specified in runtime-traversal.md.

Thus from_utf8(A, NULL, 1, nonempty_out_address) and clone(A, NULL,
nonempty_out_address) return InvalidInput in phase 1. Invalid custom callbacks
win over a nonempty owning output; a nonempty output wins over malformed UTF-8.
An inactive Option accessor with a null output returns InvalidInput, while a
valid writable output returns InvalidState unchanged. Move's existing address
before slot-state rule is an instance of this ladder, not a separate policy.
Tests combine applicable failures pairwise, preserve sentinel output bytes and
prove no later allocator callback or payload/element access occurs.

## Allocator protocol

poly_allocator has exactly context, allocate and release fields:
void *context; void *(*allocate)(void *, size_t);
void (*release)(void *, void *). Both callbacks are required for a custom
allocator; a null context is allowed. A descriptor is copied into each owner,
not borrowed. Its context and callback code must remain valid until all such
owners have been dropped. allocate returns null or fresh disjoint storage of
at least the requested size aligned to max_align_t. release receives only
non-null storage from that same allocator, once. Callbacks must return normally.

The default uses malloc/free. No request has zero size; empty collections still
have a live outer handle, with no element buffer. Growth uses checked
allocate/copy/commit/release, not realloc. All additions and size products are
checked before allocating or forming pointers. Clone may select a different
allocator; every new allocation adopts that allocator. Drop always uses each
allocation's recorded provenance, including nested values.

Each public invocation selects one effective allocator: validate a supplied
descriptor or canonicalize null to the default descriptor. Every nested direct
call, interface method, factory, clone, outcome/helper construction and temporary
traversal-work allocation forwards this exact effective descriptor. A nested
call cannot silently replace it with null, the default, an input owner's
allocator or another live descriptor. Mapping/call plans bind allocator flow
to the invoking operation, not merely to a compatible callback prototype.

An explicit public clone with allocator A and source allocated with B creates
all new storage with A; existing B storage remains unchanged and is eventually
released with B. Move and drop preserve existing provenance, and temporary work
is released with its own selected allocator. Passing the same callback functions
with a different context is not equivalent allocator forwarding. Tests inject
failure at every reached nested direct/interface/work allocation and reject
null, foreign-context and source-allocator substitutions in verified call plans.

Native stack and callback re-entry preconditions are fixed separately in
[call-stack resources](call-stack-resources.md). Returning normally alone does
not authorize unbounded foreign callback stack use or generated-API re-entry.

## Lifecycle and alias rules

For each owned T:

- clone(allocator, const T *source, T **out) returns poly_status; it makes
  independent owned storage, commits only on success and leaves source intact.
- move(T **source, T **out) returns poly_status; both slots are valid, source
  is live, out is empty, and the slots must be distinct. On success it
  transfers the handle and nulls source without allocating. Exact self-move
  returns InvalidState without modification.
- move with a null slot address is InvalidInput. Otherwise self-move,
  a nonempty destination or an empty/moved source is InvalidState, with both
  slots unchanged. Address validation precedes slot-state checks, so combined
  invalid cases have deterministic precedence. Double move is not a no-op.
- drop(T **slot) returns poly_status; a valid empty slot is a successful no-op;
  a live slot and all owned descendants are freed by the allocation-free
  iterative traversal engine and the slot is set to null. Repeated drop is safe.
  A null slot address is InvalidInput.

Distinct borrowed inputs may alias the same live value. Generated code never
uses output storage that aliases an input's owning slot or live storage.
Foreign callers must preserve that no-overlap condition and not independently
drop shallow pointer copies. Borrowed views do not survive move or drop;
they are not ownership. Read-only observers neither allocate nor transfer. A status-returning observer
writes its scalar or borrowed-view output only on Success. Reading the wrong
outcome/tag branch returns InvalidState without touching output storage.
Factories clone borrowed owned fields/elements in source order with rollback;
they do not consume callers' inputs. No public mutable field/vtable accessor
exists.

## Constants

Every public constant has an allocator-parameterized getter with the same
outcome ABI, even if its private backing data can be static const. Every read
returns a fresh independently owned value/outcome. No lazy mutable global,
process-wide initialization flag or shared owning singleton is permitted.
Constant dependencies are a checked DAG; construction and cleanup obey the
same failure rules as functions. Repeated reads, clone independence and every
allocation-failure position are tested.

## Flat interface signatures

Each implementation table contains its exact method slots, plus clone_context
and drop_context. Method slots take allocator, const void *context, the exact
borrowed arguments and the exact outcome_R **out. Private lifecycle slots are:

    poly_status clone_context(poly_clone_work *work,
                              const void *source, void **out);
    poly_status drop_context(poly_drop_work *work, void **slot);

The two work types are distinct complete private implementation aggregates,
registered by nominal identity. The clone engine carries the exact effective
allocator. These callbacks perform one authenticated task expansion and enqueue
children into that same engine; they do not recursively run public clone/drop
or invoke another traversal driver. Callback Success means task expansion was
accepted, not that an unfinished public output may be committed. Only the outer
driver commits after every queued task succeeds; failure rolls back all partial
owners. Work destinations must remain live until their task completes and may
not point to an expired callback stack local. Drop uses its allocation-free
intrusive queue. Neither TLS nor mutable global engine state is permitted.

Public clone/drop signatures remain unchanged. Exact work-type, engine-identity,
allocator-flow, destination-lifetime and one-step/commit contracts are checked
on table slots, adapters and calls. Handle move is representation-generic
and is not a vtable callback. Each table/context pair is created only by the
registered concrete-record adapter; its erased/restored identity is verified.
The table is static const private data. There is no foreign registration API.

A zero-implementation interface has an opaque handle, observer/lifecycle and
method declarations, but no constructor/table/context inhabitant. All admitted
nesting includes legacy payload-enum fields as well as records/lists/options/
results. Option None and the other Result branch remain constructible.
