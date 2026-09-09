# C17 dynamic buffers and initialized prefixes

- Status: normative for M34A-11-02D-04C-02C
- Prerequisite: the coupled numeric/memory analysis

## Typed count and requested shape

Represent an allocation's requested shape with a closed enum: one fixed object
or a sequence of elements tied to a typed count binding. The existing fixed
object path must never gain array capacity merely because spare bytes exist.
An element may itself be a complete fixed array, struct, union or scalar.

A buffer count reference is a private-field wrapper over an authenticated
immutable size_t local. Its registration creates that exact local type; checked
binding of an existing local must verify the same owner, registry and type.
The buffer-allocation builder accepts this wrapper, not an arbitrary local or
integer. Its ordinary local reference supplies the actual declaration/read and
can be the existing counted-loop bound. Namespace and declaration-dominance
rules still apply. A descriptor is a requested shape, not an allocation proof.

Lowering materializes runtime length expressions once into these immutable
bindings. This is an explicit initial admitted grammar, not a claim to accept
every equivalent C expression. A later source-length mutation cannot change the
captured count. Growth allocates another buffer with a new captured capacity.
No public constructor can assert a live extent or initialized prefix.

## Actual count/product snapshot

At the actual default allocation call, retain its authenticated graph/function/
point and original byte operand. Derive count-times-stride candidates from that
operand's actual syntax and checked immutable initializer chain. Admit pure
multiplication by checked positive constant factors, operand reversal, and
value-preserving size_t/ABI-equivalent unsigned conversions. A direct immutable
count is the stride-one case. Do not follow mutable byte locals or opaque calls
as if their current values still had an earlier initializer's shape.

Each candidate retains the actual immutable count reference, constant stride
and original numeric bounds. Every multiplication/conversion remains subject
to the existing numeric loss checks: a later guard cannot erase earlier wrap.
At restore, the candidate must match the descriptor's exact count reference and
the measured element size/alignment. Count, stride and byte evidence must agree;
same spelling, same interval, same type or a different equal-valued local is not
the original count binding. The allocation keeps this snapshot across copies,
guards and subsequent source-variable writes.

Require positive nonwrapping byte requests. Zero-length buffers use an explicit
empty/no-allocation path; do not silently treat malloc(0) as live element storage.
Maximum-count checks use the actual overflow guard before byte computation.
Unknown algebraic forms receive diagnostics rather than an invented extent.

## Storage shape and access

Dynamic element storage is an internal shape separate from CObjectType::Array.
Never fabricate a fixed maximum C array type to represent a runtime buffer.
Root, element, nested member/fixed-array and interior pointer identities retain
the original allocation. Define base-release and offset-zero behavior explicitly;
an arbitrary interior address cannot become a different allocation base.

The private storage shape distinguishes a fixed C object from an element
sequence. Restoring a nonempty buffer selects element zero; it does not create
a whole-sequence C place or convert a runtime count into an array declarator.
Base and element-zero addresses may release the original allocation. Other
interior/member addresses may not. Nested fixed arrays retain their own bounds.

Prove each offset against the original count, using actual current comparisons
or authenticated counted-loop phases. Numerical lower/upper bounds may establish
individual safe constant accesses, but an interval minimum is not a replacement
for the dynamic count relation. Unknown runtime indices require a dominating
relation to that exact captured count. Writes and lifetime changes invalidate
dependent observations; a guard over a replaced index or different count fails.

For a current runtime index, retain its authenticated storage observation as
well as conservative numeric bounds. One selected write establishes one element,
not every possible offset in the interval. Potentially overlapping writes keep
only must-initialized information. Changing an observation's source storage
invalidates dependent symbolic facts and saved-pointer equivalence; a saved
pointer must never silently follow the new index value. Unsupported symbolic
forms remain unproved rather than acquiring a fabricated exact identity.

An observed scalar binding retains its identity even when its current numeric
range is a singleton. At a join, observations of that same unchanged binding
join their possible ranges and must-initialized cells; range equality is not
required. A literal constant remains a physical offset with no index dependency.
Changing or expiring a binding conservatively retires its saved-address identity,
including an observation that happened to have a singleton range.
Two distinct current bindings, or a binding and a literal, do not coalesce into
a dependency-free constant even when both ranges contain the same single value.
Physical-offset coverage is separate from observation identity: when checking a
range against known singleton cells, count each distinct offset once and join
all cells observed at that offset conservatively. Duplicate observations cannot
stand in for an unwritten sibling element.

The same authenticated path-observation join applies to numeric facts and saved
pointer targets, not only element initialization. Different guard-refined ranges
on the same unchanged binding retain one observation with conservative joined
bounds. Numeric joins combine every reaching domain and loss history; they must
not pick one branch or discard an incomplete/wrapped observation. Actual writes
or refinements replace the current fact, while joins merge facts. Copies and
subobject overlap checks use this identity consistently. Different bindings do
not become equal merely because their ranges overlap.

Precise selected-element identity initially uses a whole size/ABI-equivalent
scalar local or parameter, including an alias resolved to that exact binding.
Complex expressions and fields can be materialized into such a binding by
lowering. An index interval without this identity supports conservative bounds
and weak updates, not a claim that every possible element was initialized.
Potentially aliased numeric observations and unseen symbolic reads retain an
unproved-write fallback. Only an exact reaching write can override it; later
comparisons cannot clear possible arithmetic history from another selection.

Count identity also has a lifetime. When its scope exits, an existing allocation
retains its original numeric snapshot but loses the right to use a later
activation of that declaration as its current dynamic bound. Re-entering a loop
scope cannot refresh this authority. A new actual allocation receives a new
snapshot through the existing release/reactivation rules.

The initial ordering grammar recognizes strict size/ABI-equivalent current
reads (`index < count`), operand reversal, and the equivalent false branch of a
non-strict opposite comparison. Counted-loop condition edges use the same rule.
Relations are intersected at joins; a disjunction or a guard on another count
cannot stand in for this witness. Redeclaring a count also retires its prior
activation even if no scope-exit edge intervenes. More general symbolic algebra
requires an explicit extension with its own proof, not an inferred substitution.

The same resolved paths feed numeric reads, writes, copies and guards. Preserve
the numeric-memory loss and overlap rules, initialization and active union
checks. Release expires every buffer/interior alias and removes its numeric and
prefix observations. Incompatible element shapes do not merge into a clean one.

## Must-initialized prefixes

A counted-loop declaration proves progress, not initialization. Derive write
coverage from actual graph actions and every continuing path. To grow a prefix,
the current element must be completely initialized before advancing the counter
or taking its backedge/continue. Nested element fields require complete coverage.
Seeing one matching write anywhere in a loop is insufficient.

Track the proved prefix relative to the actual counter and before/after-step
phase. Reads require a selected initialized element or a strict bound within
that prefix. Joins keep only common initialized coverage. Missing, conditional,
skipped or reordered writes cannot claim a complete buffer. Early break/cleanup
retains only the prefix actually established, so partial cleanup cannot inspect
uninitialized descendants. Releasing/restarting a site resets prefix evidence.

## Delivery and proof boundary

Implementation is ordered: typed count/product evidence; dynamic storage and
guarded paths; then construction/prefix proof. Intermediate stages return only
diagnostics/private evidence and keep unimplemented dynamic uses rejected.
Neither metadata nor a passed partial checkpoint can authorize rendering.

Each stage requires constructor/reconstruction/privacy controls, positive and
adversarial actual-AST cases, the cached focused/tracked/release/lint/eight-target
gates and uncapped independent review. Dynamic fixtures cover runtime counts,
zero/maximum/wrap, wrong/replaced count and stride, nested layouts, random and
counted access, skipped writes, partial construction, joins and expired aliases.
Owner transfer/cleanup contracts remain 04C-03; this document does not certify
legacy C emission or add custom/generated call contracts ahead of their stages.
