# M35-03A-05A-04C — Certified Java canonical type owner

- Status: planned
- Parent: [compiler results](M35-03A-05A-04-compiler-results.md)
- Depends on: [C owner](M35-03A-05A-04B-c-type-owner.md)
- Specification: [Java canonical owners](../../specification/typed-generation/rust-canonical-type-owners.md#java21-specification)

## Contract

Add an explicit type-owner namespace and closed publication profile for the
existing sealed Result family. Keep exact original authority and consumer scopes,
not a fake JavaSourcePackage or a copied family in each producer.

## Implementation and definition of done

1. Retain the typed descriptor in the certificate and validate exact family-only
   facade, canonical constructors, public permits and private payload storage.
2. Extend owner indexing, namespace conflict/closure checks and manifest/bundle
   inventories with the owner enum. No unchecked fallback or name-only import.
3. Preserve producer/export/consumer signature phases and constructor/accessor
   membership. Type-only imports still pay all registration/resource costs.
4. Compile two producers independently against the one original family, then a
   strict Java21 cross-producer consumer; run normal and interpreted modes.
5. Reject replacement authorities, instance/profile/owner substitutions, source
   root impersonation, foreign subtype declarations and missing type owners.
6. Measure actual classfiles/source bounds, exact/one-over capacities and unchanged
   prior outputs. Full Linux Bazel release/lint and independent GPT-6-SOL review
   precede the scoped commit/push. Compiler Result source admission stays closed.
