# C17 allocation and lifecycle evidence

- Status: normative for M34A-11-02D-04C
- Prerequisite: the 04B storage proof and actual numeric/control facts

## Actual origins, not registration claims

An allocation origin retains the actual producing call, authenticated function
and control point, requested byte extent, alignment and allocating descriptor.
The sequencing layer admits at most one root call at an actual action; nested
calls cannot hide another allocation inside an argument or cast. A graph point
number alone is not an origin. Private constructors bind it to the immutable
checked call occurrence. Equal cloned calls and CAllocationRef registrations
cannot manufacture or substitute evidence.

Borrow the numeric fact for the original byte operand, including its arithmetic
history. Require positive nonwrapping bytes before calling the allocator. A
later guard or changed variable cannot retroactively change the allocation's
extent. Default malloc alignment is the measured max_align_t guarantee, not a
host-discovered value. Custom callbacks require the exact admitted descriptor
and its actual authenticated call contract, including context.

## Pointer outcome and resource state

Fresh allocation has a known origin but may be null. Reading or copying this
known possible-null result is distinct from importing an unknown pointer.
Only its actual null/non-null test refines success; pointer type, restore and
registration do not. All copies retain the origin, while subsequent assignment
of one copy cannot change the others' pointer values.

Track possible allocation, proved null/live result and released state with
closed enums. Joins retain only established facts and separately retain any
possibly outstanding resource. A path that releases does not excuse another
path's leak. Default free accepts null or the exact unreleased default base;
custom release requires a non-null base and its original descriptor. Neither
an automatic address nor an interior/foreign pointer may be released.

Release invalidates every surviving alias, including nested aggregate copies.
Re-executing an allocation site is a new activation, never a reason to revive
old pointers. The initial raw-flow checkpoint may reject simultaneous live
activations of one site. Full construction support must handle such instances
through a proved finite ownership/work invariant rather than treating a site
as a unique runtime allocation. Every function exit accounts for outstanding
allocations, including failure paths and cleanup jumps.

## Restored type, dynamic extent and initialization

AllocationRestore preserves provenance. It requires actual success, sufficient
byte extent/alignment, a consistent effective type and exact allocator. A cast
cannot create a second allocation or initialized storage. Heap subobjects use
the same authenticated member/index identity as automatic storage, and each
dimension/offset remains inside its actual allocation.

For a dynamic buffer retain the allocated element count, concrete element
layout and the checked count-times-size relation. Prove accesses using actual
dominating comparisons or authenticated counted-loop phases. Numeric interval
minima alone cannot relate an arbitrary index to a runtime allocation count.
Writes establish fields or prefixes; reads require the selected initialized
storage and union member. Mutations invalidate dependent count/extent facts.

## Ownership is separate from initialized representation

Empty/Live/Moved/Dropped is not interchangeable with uninitialized/prefix/
complete storage. Commit requires complete owned construction. Clone creates
independent ownership; a copy retains only its existing pointer meaning. Move
transfers ownership and empties its source; drop releases once and clears its
slot. An empty public drop may succeed repeatedly without another release.

Failed construction preserves inputs and public output slots, releases only
initialized descendants and uses each allocation's recorded descriptor. The
callable ABI's ordering, alias and allocator rules remain authoritative.
Generated-body effects and iterative work summaries are discharged in 05;
neither registration nor helper names substitute for them. The initial raw
checkpoint continues to reject typed restoration and unknown generated calls.

## Proof boundaries

Implementation proceeds through raw allocation origins/lifecycle, typed heap
storage/dynamic extents, then owner construction/transfer/cleanup. Each stage
has positive/rejected controls, private-evidence compile-fail tests, complete
cached gates and uncapped review. No stage alone creates render readiness.
Stage 06 additionally tests actual generated runtime allocation failure,
sanitizers, deep values and allocation-free cleanup. These requirements are not
claims that legacy C emission already satisfies the migration design.

The raw checkpoint rejects escape into global slots, including aggregate
copies. Returning a live raw allocation cannot discharge its resource obligation
without a proved ownership-transfer contract. These explicit subset restrictions
do not claim that every rejected raw C program is invalid C.
