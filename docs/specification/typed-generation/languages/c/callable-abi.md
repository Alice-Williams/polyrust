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
errors or ordinary Result Err payloads. A transport failure leaves *out empty,
releases every partial allocation and preserves borrowed inputs. An outcome
containing a portable error is transport Success.

The allocator argument may be null to select the default allocator. The output
slot must be a valid writable address and initially null. A non-null output
is InvalidState and is neither overwritten nor dropped. Detectable null
required inputs, malformed tags or invalid views yield InvalidInput before
dereference. Arbitrary forged pointers cannot be validated by C; the foreign
caller must supply valid storage and meet documented lifetimes.

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

## Lifecycle and alias rules

For each owned T:

- clone(allocator, const T *source, T **out) returns poly_status; it makes
  independent owned storage, commits only on success and leaves source intact.
- move(T **source, T **out) returns poly_status; both slots are valid, source
  is live, out is empty, and the slots must be distinct. On success it
  transfers the handle and nulls source without allocating. Exact self-move
  returns InvalidState without modification.
- drop(T **slot) returns poly_status; a valid empty slot is a successful no-op;
  a live slot is freed recursively and set to null. Repeated drop is safe.
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
borrowed arguments and the exact outcome_R **out. clone_context takes
allocator, const void *source, void **out and returns poly_status; drop_context
takes void **slot and returns poly_status. Handle move is representation-generic
and is not a vtable callback. Each table/context pair is created only by the
registered concrete-record adapter; its erased/restored identity is verified.
The table is static const private data. There is no foreign registration API.

A zero-implementation interface has an opaque handle, observer/lifecycle and
method declarations, but no constructor/table/context inhabitant. All admitted
nesting includes legacy payload-enum fields as well as records/lists/options/
results. Option None and the other Result branch remain constructible.
