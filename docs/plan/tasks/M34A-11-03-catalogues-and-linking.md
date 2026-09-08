# M34A-11-03 — Implement C catalogues and derived header/source linking

- Status: planned
- Depends on: M34A-11-02

## Goal

Resolve every dependency, name and declaration placement from typed references.

## Definition of done

- Implement CDialect shared AST binding using the standard-call foundation from
  02D-00, and complete structural runtime signatures, constants and file metadata.
  Reuse its authoritative catalogue; do not duplicate types/effects for linking.
- Register every exact factory/observer/view/tag family in c/public-value-abi.md;
  retain actual C types for macros, enum constants and integer predicates.
- Register the complete public allocator protocol aggregate and its exact
  context/allocate/release members alongside the two public view structs.
- Derive includes, guards, forward declarations and complete-definition order from a typed dependency graph, including runtime helpers.
- Allocate names with ordinary/tag/label/member namespaces, protected standard symbols and stable generated identities.
- Authenticate complete file/prototype/member/definition inventory in both directions after helper composition.
- Reject impossible by-value cycles, linkage/definition collisions, signature mismatches and leaked private layouts.

## Shared-package integration sequence

1. Finish structural helper/declaration expansion in the authoritative C
   registry, then consume it into CFrozenRegistry. CDialect owns that immutable
   payload inside TargetAstPackage; it is not a detached verifier side table.
2. Project actual registrations into the shared generated-type/callable/value/
   file arenas in stable order. Save the private IDs returned by the existing
   builder in bidirectional C-reference bindings; never predict arena indices
   or independently author a second signature inventory.
3. Attach the completed bindings before unresolved verification. If the shared
   builder needs a consuming dialect-finalization hook, add it only at this
   unresolved construction boundary, with focused shared tests. It must not
   mutate VerifiedPackage, ResolvedPackage or RenderReadyPackage, expose private
   ID constructors, or retain interior-mutable registry/binding state.
4. Independently compare every binding, type/signature, origin and file identity
   with both authoritative inventories. Recheck after linked helper placement;
   mutually deleting a projected declaration and its binding cannot erase the
   original C registration obligation.
5. Model declaration owner separately from definition placement: an opaque tag
   registered to a public header may be completed in a private implementation.
   This does not allow a private layout to leak into public by-value signatures.

## Tests and proof

- Enforce the macro/name/typed Math dependency and public ABI identity-map
  contracts in c/platform-and-proof.md, including negative link and collision
  controls. Test-only stdio/fenv dependencies cannot leak into scalar packages.

- Exact include/placement matrices, idempotent duplicate references and unused-dependency elimination.
- Standalone public-header consumer initializes a custom allocator; missing or
  swapped members, callback signatures and incomplete public layout fail.
- Pointer-cycle positive / by-value-cycle negative; typedef/tag versus ordinary namespace collisions; same-signature wrong-owner references.
- Missing/extra/moved file and declaration inventory fault injection; repeated linking/dumps deterministic.
- Swapped same-signature projection bindings, deleted binding plus declaration,
  crossed registry brands and mutation-after-freeze compile-fail controls.
- Compare all 18 registration-kind inventory branches across independently
  branded registries and different legal registration orders.
- Separate impossible complete-layout cycles from legal lifecycle/callable
  components; pre-register prototypes and emit each specialization once.
- Integrate this distinction with shared helper closure and file-role checks:
  program-specific specializations are Implementation declarations, not baseline
  Runtime helpers that illegally depend on user declarations. Preserve existing
  missing/duplicate/forbidden-prerequisite-cycle controls; add accepted legal
  callable components without a blanket cycle-policy escape.
- Shared typed pipeline, linker, C Rust, policy and full tracked/release gates.

Test targets required by this slice must be added before it closes; proposed
future targets are not evidence of an existing implementation. All builds and
tests run in the Linux development container with pinned toolchains and normal
Bazel action/test caching.

## Commit gate

Record exact commands, counts, failures and dispositions here. Commit and push
this completed checkpoint with M34A-11-03 in the message. Keep the overall
C migration open until M34A-11-09; never substitute a partial slice for Pass.
