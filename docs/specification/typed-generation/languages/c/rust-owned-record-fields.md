# Owned record fields and partial moves

- Status: complete
- Plan: [M35-02B-03G](../../../../plan/tasks/M35-02B-03G-partial-owned-records.md)
- Foundation: [conditional owner selection](rust-owned-selection.md)

## Closed source form

A safe nongeneric Rust function takes immutable i32 parameters and returns i32.
Its root block constructs distinct parameter-anchored Box<i32> owners, optionally
moves them through local bindings, then moves them into one complete named-field
record initializer. The record is a local nongeneric struct with only Box<i32>
fields and no custom destructor. One or more field moves follow, each into an
immutable local binding. The final exit dereferences one current moved-out Box.

Do not admit record update syntax, nested record ownership, arrays, branch-local
initialization, borrows, custom Drop, unsafe, unwind or arbitrary call transfers.
Every excluded construct diagnoses rather than using a target approximation.

## Typed source identity

Retain the actual record constructor expression, compiler nominal declaration,
instantiated type, declared FieldIdx/field DefId and canonical initializer
expressions. Source field spellings are presentation only. Preserve the order
in which initializer expressions are evaluated separately from the record's
declared field order. Both matter; neither is a replacement for the other.

A field place consists of a canonical root binding and its authenticated record
field projection. MIR evidence retains the compiler Place and typed projection,
not a flattened string path. A private source capability input proves record
construction identity; a registered typed function handles that capability.
The existing scalar Box constructor capability remains unchanged.

## Ownership relation and cleanup

Authenticate each Box origin, the record's typed MIR aggregate construction and
each source-field-to-aggregate-operand mapping. Require complete field coverage,
exact operand type and move identity. Each partial move must take precisely that
declared field into its new local owner. Whole-record/field/local places cannot
be interchanged merely because their payload types happen to match.

At the final scalar read, moved-out owners are dropped in reverse local binding
order. The remaining fields of the record are dropped in field declaration
order at the record binding's exit, skipping moved-out fields. Other live local
owners retain their lexical cleanup positions. Verify actual normal-path Drop
places, their projections and order; count equality alone is insufficient.

This correspondence consumes compiler analysis and drop elaboration. It is not
a new Rust borrow checker, does not infer from names/debug spans, and does not
render compiler graph edges as gotos. Source structure remains the future target
lowering input, and all safe certificate construction remains private/checked.

## Proof boundary

The fixture and corruption matrix must establish record and field identities,
initialization/evaluation order, exact partial moves and all cleanup. Preserve
all prior compiler and native C/Java gates. This compiler-only increment does
not enable C/Java allocation or cleanup output. Scalar-record Box payloads,
nested/conditional fields and function boundaries need subsequent contracts.

## Record identity stage

G-01 first admits an individual canonical source record construction, without
claiming MIR or field-owner provenance. Require a complete named-field literal
of a local nongeneric struct, one to 128 fields, default representation and no
custom Drop. Every field must have the exact standard Box<i32> result type of
the compiler's authenticated `box_new` declaration instantiated for i32. This
includes allocator identity, not just the Box language item and first argument.

Use the checked nominal result type and compiler FieldIdx resolution, so aliases
do not acquire a new nominal identity. Retain each field DefId/type and canonical
initializer expression in source order, separately from declaration order.
The compiler's [variant metadata](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/struct.VariantDef.html)
provides declaration fields and distinguishes named-field records from tuple
constructors. Pinned compilation and fixtures establish the actual API use.

G-01 does not restrict otherwise checked Box-valued initializer expressions to
parameter-rooted local moves; that restriction belongs to whole-body G admission.
The operation input certifies identities/types only. The executable mapping
accepts this private input, not raw HIR or caller-authored field lists. The owned
program builder stores the record mapping in a distinct typed slot, with Box
construction still the required base capability and record support conditional
on that extra registered mapping. Registration order does not change support.

Rejection tests include stable unit-struct and union expressions. Allocator
identity has a separate compiler-type oracle: instantiate the authentic Box
declaration with the same i32 payload but the fixture's System allocator type,
and require the production field predicate to reject it even when initializer
and field agree. The test-only [Ty constructor](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/struct.Ty.html#method.new_adt)
does not confer checked-source status or bypass the private operation input.
It tests full type equality without enabling unstable allocator-api source.

## Aggregate and partial-move stage

G-02 admits the closed root form above through a separate query-only
`RecordOwnedBody`. It accepts a successful-analysis compiler context and body
owner, not caller-authored HIR plans, MIR bodies or source-to-place mappings.
Private construction queries canonical HIR and PostCleanup MIR itself.

The whole-body budget is at most 128 scalar parameters and 128 HIR let statements,
independently of G-01's 128-field operation limit. Constructing one owner per
field, the record and at least one extraction therefore fits at most 126 fields
in G-02; extra source bindings reduce that capacity. Exceeding either budget
diagnoses rather than dropping operations or weakening correspondence checks.

Source paths use `SourcePlace::Local(HirId)` or `SourcePlace::Field` with the
record binding, field DefId and FieldIdx. A path starts at an authenticated
parameter-rooted constructor, optionally moves between local bindings, and may
enter one record field. A moved-out field becomes another local owner. Every
source binding and its exact compiler Place remain in the evidence. Staging
temporaries are not fabricated HIR bindings: `Transfer::IntoField` retains their
actual typed place, staging assignment and aggregate assignment; `Transfer::Move`
retains an ordinary local/partial-move assignment.

Pinned Rust 1.98.0 stages local record initializers through distinct Box
temporaries in source order. The single
[ADT aggregate](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/mir/enum.AggregateKind.html)
then consumes those temporaries in declared FieldIdx order. Authenticate the
nominal DefId, variant zero, exact arguments, absent user/union annotations,
complete operands and each staging move's unique source/current owner. Both
staging order and complete source-binding event order must agree; equal Box
types or an equal aggregate operand count are insufficient.

The retained compiler [Place projections](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/mir/struct.Place.html#method.project_deeper)
include exact field index/type. One actual record storage local and every Box
local must be accounted for, including initializer temporaries. All assignments,
calls, normal blocks and drops are consumed; only independently proven unread
constant Boolean/unit compiler bookkeeping may remain semantically irrelevant.

Require the final scalar read to come from a current moved-out field owner.
Check drops after the read and before return: reverse live-local binding order,
then remaining record fields in declaration order at the record binding's
cleanup position. Every chain retains its final actual place, drop location and
canonical root drop scope. All-field moves leave no record-field cleanup.
Other constructed owners can remain outside the record and retain their own
lexical cleanup positions. Earlier local-only readers continue rejecting records.

G-02 excludes post-record allocation, inline allocation in record initializers,
whole-record moves, nested/conditional records or scopes, references, updates,
assignment/reinitialization and arbitrary call transfers. Those require later
closed source contracts. This compiler-only certificate does not enable C or
Java heap output, remove native cleanup tests, or claim universal Rust support.
