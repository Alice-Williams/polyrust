# C17 coupled numeric-memory evidence

- Status: normative for M34A-11-02D-04C-02B
- Prerequisite: fixed typed heap origins, layouts and storage paths

## One analysis state, shared arithmetic rules

At each actual control point, carry the product of initialized storage/pointer/
allocation state and numeric domains/loss history. These describe the same
incoming execution states. Do not run an optimistic memory pass and then feed
its guessed values back into a separately certified numeric pass.

Use the existing numeric engine's arithmetic, conversion, comparison, loss,
guard, counted-loop and widening rules. A private resolved-place interface may
connect that engine to storage provenance. It is not a public plug-in or a way
for callers to supply trusted keys. Standalone numeric checking retains its
existing conservative behavior without memory recovery.

## Actual locations and reaching writes

Resolve a read/write location from its actual AST and current pointer values,
selected members, array bounds and live allocation. Spelling, equal types,
descriptor registrations and stale aliases are not evidence of the location.
Numeric reads, destinations, aggregate source paths and guard terms must all
use this same authenticated resolution, not a mix of direct and indirect keys.

The private resolver also supplies actual pointer-test truth so numeric
short-circuit/conditional evaluation does not inspect a proved-unselected null
branch. Retained predicates conservatively depend on the current live memory
inventory as well as syntactic dependencies; this is not a claim of minimal
alias dependency tracking.

An exact write records the actual numeric value and inherited losses. Copies
retain those values and losses; indirect access cannot produce a fresh clean
input. Compatible scalar spellings may require a value-preserving conversion
to the actual read expression type, without erasing its numeric history.

Ambiguous or interval writes are weak updates: retain every possible prior/new
value or conservatively poison all affected numeric facts. Never install a
single optimistic value for an interval or leave a possibly overwritten exact
element unchanged. A changed union member invalidates overlapping observations.
Aggregate copies preserve source snapshots rather than resetting field defaults.

The initial implementation deliberately leaves interval numeric reads unproved
and poisons the containing root on interval writes. Exact disjoint fields retain
their numeric domains while writes invalidate root-dependent predicates/relations.

Reads additionally require initialized storage and the actual active union
member. Initialization does not justify dropping Read/Write/Arithmetic/Call/
Global history. Only the established numeric transfer rules may legitimately
produce a new bounded value, such as a Boolean comparison result.

## Guards, allocations and lifetime

Comparisons restrict current numeric observations at their actual locations.
Mutation kills dependent predicates/size relations; a guard over one alias does
not remain valid after a write through another. Releasing/restarting an
allocation removes its numeric storage facts and invalidates dependent guards,
just as it invalidates pointer and initialization facts.

Allocation bytes come from the original actual call operand at its authenticated
function/control point. They retain all arithmetic history. A later size write
or guard cannot resize the allocation or excuse an earlier wrapped calculation.
Joins combine byte ranges only for the same authenticated producing origin,
scope and alignment, retaining the minimum guaranteed allocation size.
Closed default allocation/release effects may preserve unrelated memory only
when justified by their known contracts; unknown/custom effects stay rejected.

## Convergence and certification boundary

Use monotone joins and numeric widening at actual backedges/continues. Retain
the counted-loop evidence derived from the same graph. Provisional failures
carry no successful proof and cannot silently prune the failing action away.
After convergence, strictly replay all reachable actions and verify numerical,
storage, allocation and exit obligations against their actual immutable sites.
Provisional failures poison the paired state; on a cycle the resulting strict
diagnostic may precede the original failing operation. Diagnostic priority is
not a proof of a different valid execution or permission to suppress the error.

This checkpoint returns diagnostics only. It does not create an owning-value or
render-ready certificate. Dynamic counts, initialized prefixes and buffer
construction remain 02C; incoming/custom and generated call contracts remain
their existing later stages. No rejected construct is approximated by target
source or an invented fixed array capacity.

## Required evidence

- Positive initialized heap size fields and exact indirect numeric operations,
  including alias copies, aggregate copies and compatible scalar spellings.
- Wrapped values stored/read/copied through memory remain rejected as sizes.
- Missing initialization, inactive unions, stale guards, aliased writes,
  ambiguous updates, release/reallocation and distinct same-layout allocations
  cannot acquire another location's values or history.
- Standalone numeric/index regressions, private-resolver/fact boundaries,
  complete cached build/test/lint/conformance gates and independent review.
