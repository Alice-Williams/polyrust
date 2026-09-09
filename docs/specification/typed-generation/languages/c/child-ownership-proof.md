# C17 typed children and finite ownership graphs

- Status: normative for M34A-11-02D-04C-03C
- Prerequisite: actual local leaf ownership in 03B

## Typed member roles

CMemberOwnershipRef privately retains an existing authenticated CMemberRef and
a closed CMemberOwnership enum: Required, Optional, or BorrowedMetadata. The
registry records exactly one role per existing member; conflicting registration
rejects. This is an obligation over a declaration, not another declaration,
name-based identity, allocation claim or transfer certificate.

Authenticate the original member, nominal owner and source type before alias
expansion. Preserve those references in the role. Fixed-array layers in the
member type apply the role uniformly to each terminal pointer slot, without
enumerating the bound or accepting input-authored element paths. Nested inline
aggregates obtain roles from their own actual nominal members.

| Role | Registration category | Later live-value obligation |
| --- | --- | --- |
| Required | Mutable object-pointer terminal, storable unqualified pointee | One complete exclusively owned allocation |
| Optional | Same category as Required | Null or one complete exclusively owned allocation |
| BorrowedMetadata | Pointer to const storable object, or function pointer | Authenticated static const object/function identity; no owning edge |

Pointee qualification descends through fixed arrays. Owning const slots, scalar
terminals, void/callback/FILE owners and mutable-object/void/FILE metadata reject.
The pointer slot itself may be const for metadata. Incomplete nominal pointees
may register without claiming layout or inhabitants. Owning recursive nominal
schemas are legal; actual owning cycles are not. Erased interface contexts,
dynamic buffers and incoming allocator/borrow contracts retain their explicitly
typed adapter/family/ABI obligations; this role never guesses their semantics.

Canonical inventory retains the role category, original member key and nominal
owner. Immutable package checking revalidates exact role membership/category
and the actual member definition occurrence, origin and type form. A private
reconstruction or mismatched authoritative map entry must not bypass these
checks. Public construction cannot supply live/complete/allocator/safe flags.
The registration-only checkpoint rejects storage admission when child roles
exist until actual child analysis is composed. No-role programs are unchanged.

## Actual finite graph

Extend the paired numeric/storage owner product, not an independent proof map
supplied by a caller. Allocation nodes retain original provenance and exact
physical roots. Owning locations distinguish local slots from authenticated
member/element paths within allocations. Actual source-copy/reset actions
establish edges; an arbitrary raw alias does not attach a child.

Track attached children of partially initialized, uncommitted parents too.
An active union selects its actual role inventory. Complete representation
does not imply complete ownership: every required child must be live, optional
children must be null or exclusive, metadata must have the proved lifetime,
and unannotated pointer terminals reject. Parent commit checks the entire active
inventory and allocation provenance. Duplicate parents and cycles reject.

Fixed arrays use compressed default/explicit-element reasoning. A huge optional
null array must not require bound-sized proof state. A repeated producing site
cannot stand for multiple simultaneous resources; dynamic instances and owner
prefix invariants remain the existing mandatory 03D stage.

## Transfer and destruction

Complete a child transfer only after the actual prior owning slot is cleared.
Retain 03B's same-scope single-flow adjacency until a later body summary proves
a different closed grammar. Move preserves physical allocations and numeric
history throughout the subtree, retires external aliases to every transferred
node, and restores only authenticated owning links. Do not revive arbitrary
pointer-bearing snapshots. Clone instead requires independently constructed,
disjoint allocation graphs.

Detaching a required child makes its parent dismantling/partial, not a complete
live value. Such state cannot escape or be committed; only verified cleanup
may discharge it. Release the parent only after all attached children have
been detached and accounted for. Active-arm replacement, clearing a pointer,
joins, fallback and scope exit must never hide outstanding child edges.

## Rollback and proof

At each finite allocation-failure/early/cleanup branch, retain borrowed inputs
and uncommitted outputs unchanged. Visit only actually initialized descendants,
release each exactly once through its original allocator and retain work-slot
lifetimes. The iterative runtime contract remains required for generated
bodies; finite AST proof does not claim native runtime completion.

Actual-AST positive/negative pairs cover required/optional/nested/fixed-array/
active-union roles, missing children, wrong arms, duplicate parents, shallow
copies, cycles, premature commit, stale aliases, partial destruction and
allocation failure at each finite construction point. Clone tests retain the
source while destroying the independent result. Rollback tests cover skipped/
duplicate cleanup and unchanged input/output sentinels. Private reconstruction
and join tests cannot replace those composed AST programs.

Each checkpoint runs focused, tracked, release, lint and deterministic
eight-target gates plus a fresh uncapped review. Parent 03C closes only when
all four child checkpoints satisfy the original definition of done. Existing
03D/04D/05/06 obligations are not substitutes for missing finite child proof.
