# Canonical Rust instance ownership across target packages

- Status: normative design; descriptive identity foundation implemented;
  compiler graph proof and target integration pending; Result admission closed
- Parent: [scalar results](rust-scalar-results.md)
- Integration prerequisite: [05A-04](../../plan/tasks/M35-03A-05A-04-compiler-results.md)

## Problem and selected ownership rule

An instantiated standard type is not owned by the first user crate translated.
Two producers using the same normalized Rust instance must exchange the same
original C/Java nominal type. A relay cannot repair independently certified
lookalikes by copying declarations, comparing names or replacing authorities.

Use one ordinary generated type-only package per admitted canonical instance,
per target representation profile. These packages are explicit graph members,
not a handwritten runtime or an implicit global registry. Their ownership is a
distinct typed category from a translated Rust crate. Do not create a fictitious
Rust crate root or claim to have translated all of core.

Use a private graph-scoped interner during the existing dependency-first checked
source pass. On the first authenticated encounter, derive the fixed owner from
the complete instance key and closed profile, generate and certify its leaf
package, and retain its original handle. Every subsequent encounter rechecks the
complete compiler facts and receives that same handle. First encounter selects
allocation timing only, never ownership, spelling, documentation or dependencies.
The leaf contains no consumer-dependent data. Freeze the interner after the whole
graph checks, and validate/reserve the entire graph before publishing anything.
A late conflict or failure drops all unpublished results.

Retain owned stable facts only across compiler invocations; never retain TyCtxt,
DefId/CrateNum arena identities or HIR borrows. No second compiler pass is needed
for this fixed leaf profile. The existing compiler crate-hash and pinned-path
checks are not a claim of byte-exact filesystem/toolchain TOCTOU protection.
The initial authority guarantee is within one checked graph. Separate compilation
jobs must recheck and recertify a complete merged graph; cross-process trusted
imports are outside this increment. Never deserialize an Arc address or treat a
JSON manifest as a certificate. Across runs deterministic bytes/keys agree, not
in-memory certificate identity.

## Typed source identity and evidence

Keep these concepts separate:

- `RustCanonicalInstanceKey`: descriptive, bounded, ordered source identity:
  the original generic declaration and exact normalized argument identities.
  Initial vocabulary is the closed standard Result/I32/standard narrowing-error
  instance. Primitive kind, nominal declaration and argument position matter.
- Compiler-session witness: private checked rustc type, original enum, variant
  and payload-field definitions, canonical error projection and no-drop facts.
  A key constructed by a caller cannot forge this witness.
- `TargetPackageOwner`: an enum distinguishing an actual Rust source-crate root
  from a canonical instance owner. Both retain stable identity; neither variant
  is authenticated merely by constructing this descriptive enum.
- Backend authority: an opaque reference to one exact immutable certified
  package, its checked owner descriptor and original family/member inventory.
  Equality of descriptors does not make independently certified packages equal.

Type-owner provenance retains real defining source identities and the target
representation profile. It contains no invented declaration IDs, fake source
exports, function bodies or arbitrary namespace overrides. Variant and payload
identities are checked against the compiler witness, not chosen by target names.
Unsupported instances still reject; the descriptor is not general enum support.

Retain the actual defining core crate root from rustc's crate-root identity as a
separate source anchor, joined to the instance definition's stable crate ID.
It is never the generated package's ownership key. Manifest projection has an
explicit type-owner branch carrying this anchor and descriptor, with no fabricated
RustCrateExports. Test two distinct typed fixture owners sharing that same core
anchor without admitting a second production source instance.

## Graph and naming contract

Use full typed owner keys for graph membership, duplicate/conflict detection,
closure accounting and publication. A u64 source crate ID cannot uniquely key
several instances originating in core. Source-crate and type-owner namespaces
must remain disjoint. Scope registration retains original opaque authorities,
including type-only and unused dependencies, through all relays.

For the initial no-heap scalar instance, type owners depend only on certified
target standard symbols, never on a consuming source crate. Source packages
depend on the owner; producer/relay/consumer edges remain ordinary source edges.
Reject cycles and missing owners before publication. Future instance arguments
that introduce nominal dependencies require a separate acyclicity specification;
this increment does not silently admit them.

Output spelling is deterministic data derived from the complete supported key
and a versioned target representation profile. Use a bounded injective encoding
of the admitted key, not DefaultHasher, a pointer, a visit counter or a shortened
hash treated as collision-proof. For the initial profile, key construction
requires Result and the zero-argument error declaration to originate in the same
compiler-authenticated core crate. The success argument is exactly I32. Encode
that shared crate identity as c, Result's local definition hash as r, and the
error's local definition hash as e, each exactly 16 lowercase hexadecimal digits.

- C17 representation version 1: basename
  polyrust_t1_c<c>_r<r>_i32_e<e>, with .h/.c companions. All owned symbols derive
  from this checked basename and closed member-role suffixes.
- Java21 representation version 1: namespace
  org.polyrust.generated.t1.c<c>.r<r>.i32.e<e>, with Generated.java and its ordinary
  Outcome/Success/Error nested declarations.

Angle-bracket metavariables denote the exact hex fields, not literal characters.
All represented argument identities are recoverable from this encoding; version
1 implies the fixed profile and zero error type arguments. Key validation rejects
cross-core pairs rather than dropping a different error crate ID from the name.
The representation version is a closed backend-owned enum, not a caller string.
Test worst-case identifier/path/header-guard reservations and source/type-owner
namespace disjointness. New profiles or admitted arguments need new explicit
encodings; they cannot silently reuse these names.
Changing source traversal or adding an unrelated consumer must not rename the
owner. Changed compiler instance identity or representation profile must not
reuse incompatible output paths. Existing scalar-only bundles stay byte-identical.

## Shared lowering and publication responsibilities

The compiler adapter authenticates and reconciles each encountered instance before
lowering that use, across all source/dependency signatures and admitted bodies.
The source model distinguishes scalar, unit-return and admitted nominal values;
do not put Result into RustScalarKind or confuse function returns with Result
variants. Preserve original source parameter/result types in metadata.

A graph-local binding associates each checked source instance with its exact
target owner and family/type/member handles; complete bindings freeze before
publication. Executable capability bindings use
these handles for constructors, signatures, copies and exhaustive matches.
They cannot accept a caller-supplied target type spelling or mint another family.
Rendering remains unchanged in responsibility: certified target syntax only.

Bundle preparation must validate every referenced owner, including family-only
references discovered through types, constructors and accessors. Require exact
authority agreement across the complete graph, reserve all source/metadata
bytes, then render and publish the complete set atomically. No partial files on
missing, competing or over-budget owners. Serialized metadata describes the
source instance, target profile, owned versus imported roles and original owner;
it grants no construction authority on its own.

The initial opaque error can be bound only under the authenticated Err pattern
and forwarded into Err reconstruction. Its original nominal witness is retained
even though the admitted observation carries no runtime payload bits. This does
not admit standalone error parameters/returns, arbitrary construction, equality
or formatting; those require a separately specified representation and tests.

## C17 specification

Retain the certified ordinary tagged scalar-result representation and existing
typed struct/member imports. Add an explicit closed type-owner publication
profile rather than weakening the source-crate export verifier. It requires
the selected owned result declaration and exact tag/payload roles, excludes
unrelated public declarations or executable helpers, and retains its descriptor
in the same immutable certificate used to derive original member handles.

Use a generated public header and its ordinary generated source companion to
fit the current package contract. No copied runtime source, allocator, process
state or error sentinel. The empty executable inventory has a checked zero
frame contribution, not an arbitrary caller-supplied stack bound. Header guards,
system imports and dependency includes remain structurally derived and bounded.

Replace source-root-only assumptions in dependency owner conflicts, type-only
registrations, manifests and bundle closure with the owner enum. Imported types
remain non-owning references and cannot declare or redefine the foreign tag.
Do not alter the C source-function provenance rules for ordinary crate owners.

The registered package profile has mutually exclusive source and canonical-type
variants. Retain the type descriptor in the frozen CRegistry used by projection
and its independent reconstruction, before RenderReadyPackage certification.
Recognize the strict profile before platform assertion installation. It admits
exactly one public Bool/I32 scalar-result struct with its two original registered
members, the canonical header/source pair, and required platform static asserts.
It admits no function, global value, foreign export or executable helper. The
implementation companion structurally requires its own public header.

Dependency packages expose their full TargetPackageOwner with a backend-owned
closed C representation profile. Source-only code may request an optional source
root or use a source-owner wrapper; there is no blanket root accessor returning
core for a canonical package. Dependency functions and constants require source
owners. Namespace/resource/closure checks compare full typed owner keys and exact
certificate authorities. Only source owners participate in source-crate overlap
checks. The immutable type descriptor retains core as provenance, not graph
identity. Source documentation/export logic must not impersonate a core crate.

## Java21 specification

Retain the existing sealed interface, immutable int success record and empty
error record. A generated type-owner namespace is distinct from RustCrate(u64).
Its canonical facade has exactly the selected public family and necessary
canonical constructor, with no unrelated methods, constants or source exports.
Check the explicit descriptor instead of fabricating JavaSourcePackage roots.

The existing dependency authority retains this alternative owner profile.
Nominal roles, constructors, accessors, producer/export/consumer signature
phases and frozen membership stay unchanged in meaning. Update root-keyed
closure, namespace overlap and bundle indexing explicitly; a generated type
owner is not an exception that skips verification. Measure source reservation,
classfile budgets, original owner closure and selected inventory as for source
owners. The family verifier alone is still not compiler-instance authentication.

## Proof required before source admission

Generate two producers independently against one original type-owner authority,
then a consumer passing values from each producer to the other. Compile separate
C objects and Java packages; compare Rust/C/Java native observations for both
variants. Permute source traversal and check complete output bytes and original
handle identity. Also prove unrelated consumers do not change placement.

Reject same-shaped different instances, mismatched original variant/field facts,
missing owners, two authorities for one descriptor, wrong representation profile,
forged source roots, mixed compiler facts, namespace collisions and cycles.
Exercise exact and one-over owner/descriptor/name/source/metadata limits and
producer-change invalidation/restoration. Compile-fail tests protect private
witness construction. Preserve old outputs and complete per-step Linux Bazel
release/lint and independent review before opening the compiler path.
