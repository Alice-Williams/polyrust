# M35-03A-05A-04C-02 — Strict Java canonical type-owner certificate

- Status: planned
- Parent: [Java canonical owner](M35-03A-05A-04C-java-type-owner.md)
- Depends on: [six-state family](M35-03A-05A-04C-01-java-error-family.md)
- Specification: [canonical owners](../../specification/typed-generation/rust-canonical-type-owners.md#java21-specification)

## Contract and implementation

Add a closed version-2 Java profile and deterministic namespace derived from the
complete canonical instance key. Retain all original instance/error-kind facts
and exact local family/value selections in the immutable package certificate.
Source-owner and canonical-owner metadata must be mutually exclusive. No fake
core exports or source root, first-consumer identity or handwritten runtime.

The profile accepts exactly Generated.java containing its fixed facade and
Outcome/Success/Error family, with canonical names, modifiers, constructor and
constant inventory. Reject unrelated files, functions, fields, dependencies or
caller documentation that would change canonical output. Ordinary Java source
profile checks remain intact; dependency publication rejects this owner until
04C-03.

## Layer changes

1. AST metadata: replace the source-only optional file-owner field with one
   optional `JavaPackageMetadata` enum containing Source or Canonical variants.
   Retain `JavaSourcePackage` as the source branch's existing description.
   A private-field `JavaCanonicalTypePackage` stores profile, complete original
   facts and exact local family/constant selections. The enum makes simultaneous
   source/canonical metadata unrepresentable; package-wide duplicates still reject.
2. Namespace: extend `JavaPackage` with a typed canonical-instance branch,
   containing the full instance key and closed `JavaCanonicalTypeProfile`.
   Its version-2 name is derived, never caller-supplied. Source namespaces and
   canonical namespaces remain disjoint. Preserve all existing source paths.
3. Verification: strict profile checking runs for both the unresolved package and
   independent reconstruction of resolved input. Factor the local family check
   into a read-only declaration/registration check shared by both paths; do not
   manufacture an intermediate certificate to verify unchecked input.
   Compare namespace, one canonical file/item, exact registered facade/family,
   constants, origin classes and empty dependency/documentation/helper inventory.
   Require the canonical constructor and member order. Unknown registrations
   remain errors, not unused data to silently discard.
4. Publication: derive descriptive owner/fact access only from the retained
   certificate. Keep source-only dependency publication and compiler manifests
   explicitly closed to the new branch until their separate milestones.
5. Rendering/resources: use existing ordinary structural declarations, imports,
   enum heritage and deterministic paths. Measure normal source/classfile costs.
   No runtime helper, fabricated source exports or zero-cost bypass.

## Definition of done and tests

- Exact inventory/profile/namespace/facts survive projection and independent
  certification; malformed descriptors and source/type mixing reject.
- Maximum-width keys produce valid bounded namespace/path names. Permuted
  registration and unrelated packages cannot change the generated owner bytes.
- Strict standalone Java21 compilation; measured source/classfile costs remain
  charged even without executable source methods. Test exact/one-over policies
  and owner-specific malformed inventories without inventing arbitrary limits.
- Export actual generated files; preserve old outputs/WIP; full Linux Bazel/lint,
  independent GPT-6-SOL review and scoped commit/push.
