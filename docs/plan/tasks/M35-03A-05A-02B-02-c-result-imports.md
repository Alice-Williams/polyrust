# M35-03A-05A-02B-02 — Certified C scalar-result nominal imports

- Status: planned
- Parent: [public results](M35-03A-05A-02B-c-public-results.md)
- Depends on: [owned ABI](M35-03A-05A-02B-01-c-result-headers.md)

## Contract

Retain the original aggregate/member references, resolved names, complete
defining header and immutable certificate in an opaque target witness. Import
that witness, not a copied local declaration or caller-authored layout. Derive
the catalogue again from original package authority after linking.

The existing known-type catalogue supports CertifiedDependency origins and
FixedImport policy. First prove whether a language-owned referenced-type enum
(standard scalar versus certified aggregate) can use that path. Do not add a
parallel generic type-import framework without a demonstrated missing contract.
Member spellings must likewise come from the original certificate before
rendering, never from text matching or independent consumer name allocation.

Read-ahead confirms that `KnownFieldSpec` plus `DependencyPolicy::Member` can
carry an authenticated aggregate owner and field spelling without occupying the
ordinary import namespace. Select the owner's known-type reference structurally
as well, so member-only uses still introduce the correct defining header.
Generated public members already have typed owner scopes from 02B-01.

Synthesized target result structs need not be direct Rust type exports. Enumerate
their actual certificate-owned `CStructRef` witnesses and select them through
typed signatures/references; do not invent Rust declaration IDs or find them by
names. The compiler's canonical Result-instance ownership remains a separate
source-integration obligation. Keep standard scalar and certified nominal
known-type variants explicit; no arbitrary known-type strings are admitted.

## Implementation boundaries

1. Derive an opaque aggregate witness from the original certificate's actual
   header declaration and resolved type/member bindings. Retain its complete
   layout and certificate authority. Same spelling or layout is not identity.
2. Keep imported aggregates separate from owned registry declarations. Prefer
   retaining original foreign aggregate/member references so signatures and
   relay packages cannot accidentally manufacture a second nominal type.
   Membership and layout reads require the registered witness. Imported
   definitions remain immutable: adding members, redefining the aggregate or
   emitting its declaration in a consumer must fail. Registration failures
   must leave every registry inventory unchanged.
3. Extend the language-owned referenced-type vocabulary with standard and
   certified variants. Reconstruct its known-type catalogue from the consumer
   registry's immutable authorities. Use the existing fixed-import path and
   the C tag namespace for aggregate names; standard typedefs remain ordinary
   identifiers. Select imported types and members from structural file uses.
4. Resolve member spellings from the authenticated original type witness at
   link time, retaining typed member identity. Do not allocate consumer field
   names or look them up by strings in the renderer. Reference discovery must
   account for both member reads and aggregate initializers.
5. Reconcile entire included headers, including unused public types, across
   direct and transitive dependencies. Tag and ordinary namespaces stay
   distinct. Preserve the original header through a relay rather than copying
   a declaration or inventing an alias header. Charge closure/layout/name/frame
   costs before rendering and prove that type-only imports are not invisible.
6. Only after the above gates pass, replace the temporary aggregate-header
   publication rejection with exact certificate-derived API collection. Admit
   local/imported result signatures through the same registered shape proof.
   Keep compiler source capability admission separate from this target change.

The implementation must audit ownership-sensitive registry operations, not just
relax the central membership check: a reference valid for reading an imported
layout is not permission to mutate or declare that layout locally.

Reconcile all type names introduced by the complete dependency header, even
for scalar-only imports. Preserve C tag versus ordinary namespaces, original
owner through relays, transitive header dependencies, frame/layout bounds and
atomic registration. Reject conflicting nominal authorities rather than
treating same-shaped types as interchangeable. Source Result-instance identity
remains a separate compiler obligation.

## Definition of done and tests

Producer/relay/consumer packages certify independently, preserve exact original
types and fields, and agree natively across GCC14/Zig and optimization levels.
Type-only imports and mixed scalar/result APIs work. Cross-namespace equal names
behave correctly; same-namespace collisions, forged or missing authorities,
renamed members, header conflicts/cycles, missing transitive types, wrong
signature owners and over-budget imports reject. Mutation controls show that
metadata alone grants no authority. Full Linux release/lint, preserved existing
outputs/WIP and fresh review precede commit/push and removal of the temporary
aggregate-header dependency guard.
