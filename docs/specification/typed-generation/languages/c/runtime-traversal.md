# C17 depth-independent runtime traversal

- Status: normative migration target; owner M34A-11-06
- Separate from generation/compiler resource budgets in Stage 04

## Admitted values and execution stack

Recursive nominal/interface type graphs can contain arbitrarily deep finite
values created through the public factories. The backend must not reject those
shapes or translate value depth into unbounded native C call depth. Clone,
construction/coercion, drop, semantic equality, list searches and portable-test
expectation comparison use explicit iterative work engines. Interface callbacks
participate in that engine; recursively re-entering public clone/drop/comparison
drivers through a vtable is prohibited. There is no fixed semantic nesting cap.

The exact private clone_context/drop_context prototypes in callable-abi.md
receive distinct nominal clone/drop work pointers. They expand one task in the
active engine, never hide it in TLS/global state. A callback's destination slot
must outlive queued work; outer drivers alone commit complete public results.

Generated owners have finite exclusive owning children; borrowed aliases are
read-only and do not create additional ownership edges. Public factories clone
their borrowed inputs, so they cannot create owning cycles. Arbitrary forged
foreign storage and shallow copied owning pointers remain outside the ABI.

## Allocation-free destruction

Drop must succeed without allocating for every valid live or partially built
owner. Do not require a fallible work-stack allocation during rollback.
Each owned representation includes private intrusive pending-work bookkeeping.
The finite program specialization inventory supplies a closed tagged union of
exact concrete owner pointer types; pointers are never recovered through
container-of, prefix-layout casts or untyped identity lookup. Selected union
members have the same active-tag proof as other generated unions.

Before releasing a popped owner, detach/enqueue only its initialized owning
children, retaining each allocation's recorded allocator. The pending link
belongs to the child being queued; no link may reference freed parent storage.
Live immutable values do not expose this bookkeeping; it is mutated only under
exclusive destruction ownership after all borrows have ended. Callback release
must return normally, as required by the allocator contract.

Maintain checked owned-node counts during construction/clone, including each
live outer handle and initialized children. The verifier proves work visits
are bounded by this count, each owner is queued/released once and no traversal
counter wraps. Count overflow returns Capacity before committing a new owner.
This is representational size accounting, not a nesting or frontend arity cap.
Partial rollback uses its actual initialized-node inventory, not a claimed
fully initialized count. Empty/moved slots retain the existing lifecycle rules.

## Fallible construction, cloning and comparison

Construction/clone work items carry typed source/destination identities,
selected tags and initialized-prefix state. Allocate their explicit work
storage through the selected allocator with checked size arithmetic. On
AllocationFailure or Capacity, release work storage and all partial owners
through the allocation-free destruction engine; preserve inputs and outputs.
An interface task carries its exact witness/table binding, not a guessed cast.

Comparison uses separate read-only work items, not intrusive mutation of its
borrowed inputs. The semantic IEEE relation and NaN-class portable-expectation
relation retain separate closed identities. Heap work storage introduces ABI
AllocationFailure/Capacity, never a portable computation error or a false
comparison result. List Contains/IndexOf use that same semantic comparator and
propagate its transport failure rather than treating it as a mismatch. Release
all temporary work on match, mismatch, short-circuit and failure. Scalar/text/
byte direct comparisons remain allocation-free where their explicit plan has
no work-storage operation.

PortableTests treats comparator transport failure as a failing harness result;
it must not satisfy an expected portable error or increment successful-case
counts. New transport effects do not infer a fictitious portable capability:
each owning mapping's C plan and wrapper authenticate them explicitly.

## Required evidence

- Recursive I / R(child: Option<I>) and comparable Node(next: Option<Node>)
  values, built through ordinary public constructors, not forged layouts.
- Deep finite chains with equal/unequal leaves; nested F64 NaN-class versus IEEE
  comparisons and exact signed-zero/representation audits kept distinct.
- Clone/coercion/drop and list search with native call depth independent of
  value depth; instrumented work/visit counts and a small-stack native run.
- Allocation failure at every reached construction/clone/comparison work and
  owner allocation; no input changes, unchanged output sentinels, zero leaks.
- Drop and rollback with an allocator configured to reject every new allocation:
  no allocation callback occurs, and every initialized allocation is released.
- Exact checked-count arithmetic boundary/one-over tests without allocating an
  impossible SIZE_MAX-sized fixture; mutation cannot forge owner count facts.
- Wrong work tag/type, duplicated queue entry, freed-parent link, skipped child,
  reordered initialized-prefix and recursive callback re-entry mutations.
- Missing/swapped work-parameter prototypes, a different active engine, expired
  callback-local output destinations and premature public commit mutations.
- Both pinned compilers/optimizers plus ASan/leak/UBSan and full cached gates.

These are obligations, not claims that the legacy runtime already meets them.
