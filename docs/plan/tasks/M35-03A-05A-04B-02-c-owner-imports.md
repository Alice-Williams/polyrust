# M35-03A-05A-04B-02 — Typed C dependency ownership

- Status: complete
- Parent: [C type owner](M35-03A-05A-04B-c-type-owner.md)
- Depends on: [type certificate](M35-03A-05A-04B-01-c-owner-certificate.md)
- Specification: [canonical owners](../../specification/typed-generation/rust-canonical-type-owners.md#c17-specification)

## Contract and implementation

Reconstruct separate source/type-owner API inventories from immutable certificates.
Type owners publish only the selected original struct/member handles, not functions,
constants or fake source exports. Retain exact Arc certificate authority independently
of descriptive owner equality. Replace blanket root access with the full typed owner
and optional source-only root access; core provenance is never a package key.

Audit registry conflicts, namespace overlap, dependency symbols and resource closure.
Use full owner keys throughout, allowing different instances from the same core.
Source-crate overlap checks apply only to SourceCrate. Compiler paths not yet ready
for type nodes must reject explicitly, not collapse them onto core or silently omit
them. Full compiler bundle support is 04D.

## Definition of done and tests

- Same certificate reuse succeeds; equal-descriptor replacement authorities fail.
- Two distinct fixture instances sharing core remain distinct registered owners.
- Exact type/member imports, unused registered owners and relay closure are retained.
- Source/type namespace conflicts, missing owners, wrong profiles, forged source
  roots and resource overflows reject atomically.
- Existing source-owned API and outputs remain unchanged.
- Full Linux Bazel release/lint and independent GPT-6-SOL review precede scoped
  commit/push. Native cross-producer adversarial proof follows in 04B-03.

## Concrete integration audit

04B-01 deliberately leaves dependency publication closed. Update these existing
source-only assumptions together when this step begins:

- `dialect/shared/dependency_api.rs` and its inventory/constants/structs modules:
  retain the full typed owner in the original authority. Canonical inventory
  contains exactly its certified struct/member roles, without functions,
  constants, aliases or inferred Rust source exports.
- `ast/registry/dependency_owners.rs`: compare full owner keys and exact
  certificate authority. Distinct instances in core are not duplicate source
  owners. Preserve separate header-spelling and symbol collision checks.
- `dialect/shared/dependency_symbols.rs`: apply source-crate overlap only to
  source owners; canonical owners still participate in type/header checks.
- `resources/dependencies.rs`: key closure by full owner and retain certificate/
  edge budgets, unused registered imports and independent original measurements.
  Preserve existing zero-frame constant-only source packages; additionally accept
  the strict type-only certificate's measured zero. Never give an executable
  source function a zero cost merely because it uses an imported type.
- Compiler `c_graph`, `api_manifest`, `output/bundle` and C constant-import
  bindings: replace blanket package-root assumptions with checked source-only
  access. Until 04D implements type nodes/manifests, reject canonical owners
  explicitly rather than returning core as a fake source root.

General dependency packages/APIs expose `owner()` and optional `source_root()`.
Callable/constant proofs remain constructible only from the source branch.
Canonical core provenance remains in the type descriptor, not the graph key.
Never substitute a default root for a missing source owner.

Exercise distinct same-core instances, cloned original versus independently
recertified equal descriptors, unused imports, relays and source-owner conflicts.
Existing scalar manifests must remain byte-identical; canonical manifest emission
is outside this checkpoint.

## Implementation and verification

The dependency API now publishes canonical records from their strict certificates,
with `owner()` and optional `source_root()`. Original Arc authority remains
separate from the descriptive key. Direct and transitive owner maps use full
typed keys, with a separate source-crate collision guard. The complete original
edge view includes unused imports; public signature exports remain narrower.

The dedicated `c_canonical_dependencies_test` covers nine cases: original roles,
same-core distinct instances, clone versus replacement authority, foreign
declaration denial, nonzero executable costs with type-only imports, record
relays, unused scalar diamonds, source-root/hash conflicts and atomic header
collisions. All nine passed in the Linux container on tree
`abc0f72d66501a3c7da2d631b7f46d5e1ff58328`
(invocation `2a734017-0034-45cf-b178-5628b7164ba9`). An earlier run exposed
missing documentation ancestry in the source test fixture; that fixture was
corrected, without weakening production validation.

Independent review identified missing direct manifest rejection evidence.
`c_canonical_owner_manifest_test` adds a valid source-only positive control,
an otherwise-valid source certificate with an unused canonical import rejected
at both manifest construction and owner revalidation, and rejection of a
canonical certificate masquerading as its core source root. Both cases pass
on code tree `67c66a93e4ae0bf1c17e70664a6a8033ff70ed42`, together with the
nine dependency cases, Rustfmt, Clippy and Buildifier (invocation
`5f856665-3bac-4194-aa9c-c93a91e07598`, 21.931 seconds).
The first manifest test build exposed a missing direct dependency on the
existing workspace binary64 crate; the target now declares that dependency.

The earlier integration tree `abc0f72d66501a3c7da2d631b7f46d5e1ff58328`
passed all 1,053 Linux release/lint targets (133 executed, invocation
`cdcab148-4cd4-45a9-a2ce-55978e3e6a4b`, 1,399.822 seconds). That run includes
all 886 selected C unit cases, with zero failures/ignored cases and the eleven
existing native partitions still tested separately. All 530 recorded generated
files remain byte-identical and all 45 protected WIP hashes are unchanged.

Both independent GPT-6-SOL reviewers find the final code tree clean. Their one
required evidence gap was the manifest test now added and passing; no core
finding remains.

The expanded code tree passes all 1,054 Linux release/lint targets, including the
two new manifest cases: invocation `dc98a73b-3b1a-43ea-af34-a605115d59f5`,
498.989 seconds, 70 targets executed and unchanged results cached. Rustfmt,
Clippy, Buildifier, compiler contracts and all native partitions pass.
The final documentation-only checkpoint receives the same release command
before commit, and the recorded output/WIP comparisons are repeated.

This completes typed C dependency ownership, not native six-state composition,
Java type owners, canonical compiler bundle publication or Rust Result source
admission. Those remain separately gated; no legacy runtime removal is approved.
