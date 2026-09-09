# C17 owner contracts and actual lifecycle proof

- Status: normative for M34A-11-02D-04C-03
- Prerequisite: completed allocated-storage and initialized-prefix proof

## Requested roles are not evidence

An ordinary C pointer can be a borrow, construction temporary or owning slot.
Neither its spelling nor a shallow copy determines semantic ownership. Register
an owning local role with a private-field COwnerSlotRef over an existing exact
CLocalRef. The registry authenticates its function, lexical scope, source type
and membership. Do not create another name-based ID or a second C declaration.
Canonical inventories expose this role and its actual local owner.

The initial role requires a mutable object-pointer local with an unqualified
object pointee. Void, callback, stream, scalar, array-of-pointer and const/borrow
slot categories cannot be relabelled as this role. Authenticate aliases before
canonical checks; preserve the original local/type references for dependencies.
An incomplete nominal pointee may be registered, but registration does not
prove its complete layout, initialization or valid inhabitants.

Revalidate role shape/membership with the immutable package, including the
actual local declaration occurrence. A foreign or reconstructed inconsistent
reference must fail. No public role constructor accepts live state, an allocator
claim, prefix completeness, clone identity or a transfer-success flag. The
registration-only checkpoint explicitly rejects owner safety admission until
actual lifecycle analysis is integrated; it cannot silently ignore annotations.

## Private state and actual actions

Missing/uninitialized slot state is distinct from Empty, Live, Moved and Dropped.
Representation zero establishes an empty slot, not an inhabited owner. Live
retains the actual successful allocation, checked complete payload and original
allocator provenance. Different possible resources and unfinished transactions
remain obligations at joins; uncertainty is never converted to Empty.

Derive transitions from the same immutable graph and incoming numeric/storage
snapshot as existing analyses. Run the ordinary actual action and establish
private lifecycle evidence only when every relevant transfer succeeds. Solver
fallback may forget values but cannot erase outstanding ownership obligations;
strict replay checks every reachable action and crossed scope/exit.

The first local implementation admits complete pointer-free fixed payloads,
including scalar and composed fixed representations, allocated with the already
authenticated default allocator. Pointer-bearing children, dynamic families,
incoming owner parameters and unresolved calls stay rejected until their own
proof checkpoints. This subset does not discharge the complete owner milestone.

Claiming a local owner requires an actual live whole allocation base, matching
type/layout/allocator, complete initialized representation and exclusive claim.
Neither an interior pointer, second restore nor independent shallow alias is a
new owner or clone. Retain the allocation's numeric history and physical state.

Move requires a live source, distinct empty destination and actual source reset.
The initial local grammar recognizes a destination copy followed by the source
null assignment without an intervening observable action or control choice.
An in-flight copy is not completed ownership: any use, escape, conflicting write,
scope exit or unmatched join rejects. Do not invent a trusted high-level move
flag. More general generated/public call sequences are authenticated by body
summaries in 05. Invalid/self moves preserve both slots under the callable ABI.

Move preserves the live allocation but retires old borrowed views and aliases;
drop retires the allocation itself. Actual release must use the original base
and allocator and must be followed by clearing the owning slot. An already empty
drop may succeed without releasing anything again. A raw alias release cannot
erase the remaining slot-reset obligation. Every exit and redeclaration must
account for live owners and pending resets before retiring local state.

For the local leaf grammar, writes to owner slots use the direct local AST node;
indirect slot writes reject. A move source is likewise a direct local read. Copy
and reset are connected by one unconditional flow edge, without a crossed scope
or another predecessor at the reset. A local drop uses the closed default release
call with an object-to-void conversion of the direct owner read. This grammar
does not infer a transaction through an arbitrary borrow, conversion or helper.
Claim uses an already bound allocation with a structurally pointer-free payload;
even inactive pointer-bearing union alternatives require the later child proof.
After each successful ordinary operation, recheck every live leaf owner's actual
slot/base and complete active payload. Switching a union arm cannot preserve an
outdated inhabited-owner claim over a partially initialized replacement. Build
partial replacements in non-owning construction storage before committing them.
Only the exact owner awaiting a verified release-reset is exempt while its
physical allocation has already been retired; no other action may intervene.

## Nested construction and rollback

Define typed child roles using authenticated nominal/member/element paths,
separating required or optional owned children from borrowed allocator/table
metadata. No field-name convention grants ownership. Active union/variant state
selects the actual child inventory before a payload access. Complete memory
alone is not complete ownership; required children must be live and complete.

An owned child has one exclusive parent or local/work owner. Transfers detach
the previous owning slot; borrowed aliases do not create edges. Reject shallow
owning copies and owning cycles. Clone requires independent actual allocations
and recursive owned construction, not pointer inequality or a helper name.
Commit checks the complete active child inventory and allocator provenance.

Failure leaves borrowed inputs and uncommitted outputs unchanged. Rollback
traverses only established fields/elements/children and releases each with its
recorded allocator. Clearing a parent pointer cannot erase its children from
resource accounting. Work destinations must outlive their pending operation.
The iterative runtime/ABI contracts remain authoritative for generated bodies.

## Dynamic allocation families

A producing call occurrence is not a unique runtime allocation across loop
iterations. Extend private evidence with finite family/instance relationships
derived from actual construction counters, successful producing calls and
exclusive owner destinations. Do not overwrite one singleton origin, revive
earlier aliases or drop outstanding instances at a join.

Prove the ordered initialized-owner prefix and its exact outstanding resource
inventory together. Each continuing construction path must account for its
fresh allocation before advancing; failure retains only the completed prefix
and current partial child. A cleanup/work invariant visits each live instance
once, with checked progress/count arithmetic and the original allocator. Early
exits and skipped/duplicated child work cannot satisfy completion. Finite
analysis must not enumerate every runtime instance or impose a semantic depth
cap. The runtime traversal specification fixes the later work-engine behavior.

## Proof and integration boundary

Every checkpoint has positive actual-AST programs, rejected mutations, private
evidence/constructor tests and the full cached Bazel gates. Local tests include
partial commit, missing reset, double move/release, self move, stale views,
branch joins, early returns and cleanup jumps. Nested/family tests add required
children, wrong active payload, failure prefixes, multiple simultaneous live
instances, exact cleanup counts and unchanged output sentinels.

Independent uncapped review checks the composed implementation, not only the
latest repair. Parent 04C closes only after all owner obligations pass. Incoming
custom allocator/borrow boundary proof remains 04D; generated-body effects and
helper SCC summaries remain 05; native fault injection and iterative generated
lifecycle proof remain 06. No intermediate diagnostic or role registration can
render C or advertise an unimplemented Supports mapping.
