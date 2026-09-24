# Rust scalar results in C17

- Status: private transport and owned public ABI complete; nominal imports in progress
- Contract: [shared](../../rust-scalar-results.md)

Use a source-derived complete struct with a typed Boolean success tag and I32
payload. Initialize both fields on every construction; canonical error
construction sets the unused payload to zero. Any error-tagged payload is
ignored semantically. Avoid unions, uninitialized storage and error sentinels
that collide with valid integers. By-value transport needs no custom allocator.

The result's nominal identity, member identities and complete defining header
must survive independent package certification/import. Extend the currently
scalar-only public signature profile narrowly; arbitrary aggregate, recursive,
pointer-bearing or heap types stay rejected. Update call/frame/metadata/output
budgets before admitting the shape. No unchecked C struct/name reconstruction.

Source match lowering tests the tag before exposing success payload or the
opaque error witness; it evaluates only the selected arm. Foreign C callers
must use the published nominal type. Fully initialized representation does not
authorize observing an inactive payload in the source model. Prove standalone
headers, native ABI/copies/returns and original dependency ownership.

## Private transport boundary

Before public ABI support, the certified profile may pass/return this exact
unqualified two-field layout only through internal functions in its defining
implementation file. The complete declaration must precede signatures. Layout
recognition uses registered nominal/member types, never spelling. The closed
call-effect analysis can include these pointer-free values, but still derives
effects from actual bodies and their acyclic callees. This does not certify Rust
variant identity or make all same-layout source types interchangeable.

## Owned public ABI boundary

The certified profile admits only the same exact complete Bool/I32 layout
as an owning public-header declaration before prototypes. Its type and fields
are exported graph symbols because C exposes fields in a complete struct.
Source variant access remains independently checked during future HIR lowering.
External signatures use header-owned types; internal functions may reuse them.
No standalone source-only aggregate ABI or private type leakage is admitted.

Exported fields retain aggregate scope, not the ordinary package-wide namespace.
The linker derives `BindingScope::Type` from each exact registered member owner
in the original checked projection and reconstructs it during certification.
Different structs may reuse field names, including a name also used by an
ordinary function. No textual owner prefix or renderer-side name repair is
permitted. This extension leaves existing source-private spelling allocation
unchanged.

## Certified nominal import boundary

`CDependencyApi` enumerates opaque `CDependencyStruct` witnesses from the actual
owned public declarations in its immutable certificate. Each witness retains
the original `CStructRef`, ordered members, resolved type/member names and
defining header authority. A consumer explicitly registers that witness before
registering a function whose signature uses it. Registration retains the original
nominal identity; no consumer-owned surrogate or same-layout substitution exists.
Registration and subsequent membership reads independently reconstruct the
exported layout and spellings from the original certificate. Cached summaries
alone cannot authorize altered names, members or owners.

Imported layouts are readable but not owned. They cannot be extended, redefined,
forward-declared or emitted as local aggregate declarations. Failed registration
must not mutate any registry inventory. Missing type witnesses, wrong member
owners and independently certified lookalikes reject before rendering.

The C plugin uses the shared catalogue rather than a parallel import framework:

- `CReferencedType::Standard` carries standard typedefs in the ordinary namespace.
- `CReferencedType::Certified` carries nominal witnesses in the tag namespace,
  with certificate-derived `KnownTypeSpec` and `FixedImport` of the original header.
- `CImportedMember` carries the typed original owner/member pair. Its
  `KnownFieldSpec` uses `DependencyPolicy::Member`; field references select the
  owner type as well, without creating free-standing ordinary imports.

Structural uses determine each file's selected references and includes. The
renderer receives exact resolved names and performs no string lookup, field
renaming or layout reconstruction. A relay publishes original foreign witnesses
needed by its public signatures, not copied definitions or invented alias headers.
Private and unused foreign registrations are not part of the published type API;
they remain in the immutable package registry for safety and resource checks.

Dependency reconciliation covers the entire direct/transitive header closure,
including unused tags introduced by scalar-only APIs and type-only imports.
Reconstruct public tags from declarations, preserve separate C namespaces, and
reject conflicts in one namespace, conflicting certificates for one crate,
output/header collisions and source-identity cycles. Existing bounded traversal,
layout and frame policy remain mandatory. Foreign call costs compose from the
original certified package; imported type registration does not copy its frames
into the consumer's owned function inventory.

Before the import checkpoint is complete, prove independently certified
producer/relay/consumer packages with strict GCC/Zig compilation, mixed objects,
both optimization levels, same-compiler UBSan, type-only construction/member
access and compiling tag/payload faults. This target contract does not yet admit
Rust Result source operations or establish source variant identity.
