# M35-03A-05A-04B-02 — Typed C dependency ownership

- Status: planned
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
