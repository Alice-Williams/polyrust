# M35-03A-05A-04C — Certified Java canonical type owner

- Status: in-progress
- Parent: [compiler results](M35-03A-05A-04-compiler-results.md)
- Depends on: [C owner](M35-03A-05A-04B-c-type-owner.md)
- Specification: [Java canonical owners](../../specification/typed-generation/rust-canonical-type-owners.md#java21-specification)

## Checkpoints and dependency order

1. [04C-01 — six-state local family](M35-03A-05A-04C-01-java-error-family.md):
   exact sealed interface, success record and six-constant error enum. Keep the
   existing payload-free local profile separate; do not reinterpret its proofs.
   Complete: all 1,057 release/lint targets, 16,406 native observations,
   compiling wrong-kind/payload controls and two clean independent reviews.
2. [04C-02 — strict type-owner certificate](M35-03A-05A-04C-02-java-owner-certificate.md):
   closed versioned namespace, full original facts and exact family-only facade
   retained in the immutable certificate. Source-only dependency APIs still reject.
   Complete: all 1,058 release/lint targets, eight strict owner cases, measured
   four-classfile budgets, exact/one-over metadata counts and clean reviews.
3. [04C-03 — original typed imports](M35-03A-05A-04C-03-java-owner-imports.md):
   full owner keys, original enum-constant/family/member authority, consumer-scoped
   signatures and complete unused/diamond closure accounting.
4. [04C-04 — cross-producer native proof](M35-03A-05A-04C-04-java-owner-proof.md):
   independent producers, generated consumers in both directions, all six error
   values, positive/native negative controls, deterministic output and capacities.

Each checkpoint has its own full Linux Bazel/lint gate, GPT-6-SOL review loop,
commit and push. No checkpoint alone opens compiler Result source admission.
04D publishes the complete C/Java graph; 04E binds checked HIR operations.

## Contract

Add an explicit type-owner namespace and closed publication profile for the
sealed Result family with a six-constant immutable Error enum (version 2), not the
earlier payload-free Error record. Keep exact original authority and consumer scopes,
not a fake JavaSourcePackage or a copied family in each producer.

## Implementation and definition of done

1. Retain the typed descriptor in the certificate and validate exact family-only
   facade, success canonical constructor, public permits and six Error constants.
   Exclude custom enum bodies, fields and methods; keep foreign null rejection.
   Bind the full 04A-03 error-kind inventory; test kind collapse and substitution.
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
