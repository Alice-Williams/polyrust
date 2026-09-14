# Conditional owner selection

- Status: complete
- Plan: [M35-02B-03F](../../../../plan/tasks/M35-02B-03F-conditional-owned-selection.md)
- Foundation: [early owned returns](rust-owned-early-returns.md)

## Source contract

Retain the existing safe nongeneric Rust function signature with exactly one
immutable bool parameter, immutable i32 parameters and an i32 result. The root
block constructs one or more distinct parameter-anchored Box<i32> chains and
optionally performs whole-owner moves. Its final binding has an initializer
`if flag { first } else { second }`; each arm contains only a direct path to a
different current owner. The destination binding is immutable. The final exit
is `*selected`, `return *selected` or `return *selected;`.

The source input retains canonical constructor, condition, branch, arm operand,
destination binding and exit identities, plus actual lexical containment. Both
arm transfers initialize one root-scope binding, rather than creating fabricated
source bindings for the compiler's branch temporaries.

Unsupported adjustments, destructuring, branch statements or allocations,
additional conditions, repeated constructor anchors, records, loops, unsafe,
custom Drop and unwinding must diagnose. Prior ownership readers keep their
original closed source contracts.

## Compiler relation

Each user-guard outcome defines a complete executed MIR path. The user guard is
authenticated against the actual source Boolean parameter. Any later switch
must be a compiler cleanup decision with a known Boolean value derived from
the latest executed admissible definition; uninitialized, nonconstant, projected
or ambiguous producers reject. Keep typed evidence for the deciding definition,
switch location, Boolean value and selected edge. Do not assign authority merely
because a local looks like a drop flag.

Cross-check each resulting path against the canonical source transfer: identify
constructors by compiler declaration and instantiated types, anchor them to
distinct source parameters, preserve prefix moves and map the selected chain
into the common destination. A MIR local can represent different origins on
mutually exclusive paths, never two simultaneous owners on one path. Account
for every ownership instruction and each compiler-only constant/copy operation.

The selected destination is read before cleanup. It is dropped exactly once;
every other live owner is dropped once in reverse lexical binding order. The
selected moved-from binding has no remaining cleanup obligation. Check actual
drop places, order and normal Return, independently of the propagated flag
values. Incorrect compiler-flag substitutions must not be able to justify
incorrect ownership cleanup.

Require bounded complete path/edge coverage and explicit treatment of any
compiler-impossible branch before admitting the shape. No malformed or unknown
edge is silently discarded. The implementation must document its exact pinned
MIR inventory and any narrower restriction discovered during observation.

## Pinned implementation

Rust 1.98.0 PostCleanup represents the admitted selection with one shared
destination local, one source guard and exactly two cleanup switches. Each
cleanup switch copies an unprojected, non-parameter bool local. Its latest
executed definition must be an evaluated Boolean constant; copying another
flag into it, using a constant discriminator directly or an unknown definition
is outside this first contract. Source-guard producer copies remain supported
by the existing parameter relation, not by the cleanup-flag resolver.

Both complete paths contain the same two distinct cleanup-switch locations and
locals, with complementary Boolean outcomes at each switch. Thus each edge of
both cleanup switches is actually covered; their targets must be distinct and
the union of both path inventories must equal the complete body. No impossible
edge exception is needed for these pinned shapes. Every path is acyclic and
bounded by the existing 512-block/1024-local inventory limits.

The global flag visitor permits only verified constant stores, exact copies at
those certified switch locations and storage markers. All other flag uses
reject. Only executed flag assignments enter each path's consumed-instruction
set. The independent owner-chain relation still checks all calls, assignments,
read and drops. Exactly the selection transfer follows the user guard; every
other owner move and constructor precedes it. Cleanup switches follow the
selected owner's drop. All source binding-to-MIR-local mappings agree across
paths, even though the shared destination has a different constructor origin.

`SelectedOwnedBody`, `SelectionPath` and cleanup `Decision` are separately
private evidence types. A selected path cannot become a complete linear body,
and a selected body cannot impersonate the earlier if/else-return grammar.

## Evidence and boundary

Safe construction queries analyzed compiler data. Arbitrary source claims or MIR
substitutions remain private test inputs. Store complete-function evidence
separately from path evidence; retain canonical types and existing executable
constructor-capability bindings. Rendering stays structured HIR-based; these
MIR paths do not become a goto renderer or a replacement Rust borrow checker.

Tests prove both selected chains and remaining cleanup, typed construction
restrictions, nonempty inventories and deliberate relation corruptions. This is
compiler-only correspondence. Allocator policy, native failure/cleanup behavior
and C/Java heap generation remain later milestones.
