# M34A-11-02A — C registries and nominal identities

- Status: in-progress
- Depends on: M34A-11-01, M34A-11-01R

## Goal

Implement this bounded part of M34A-11-02 without introducing a raw-source path
or advertising capabilities before their mappings exist.

This registry-only foundation may proceed while the amended C ABI is reviewed
and Java's hosted CI finishes. It introduces no context-verified AST package,
ABI lowering or certificate. Those boundaries remain gated on M34A-11-00R and
M34A-10AB at the next slice; review findings still apply before integration.

## Definition of done

- Add private registry-scoped, kind-specific identities for struct, union, typedef, enum, enumerator, function, object, member, parameter, local and file registrations. References retain origin, owner and complete structural type/signature.
- Register exact interface adapter/witness/table identities and function-owned loop, switch, cleanup-exit and allocation identities for later AST/proof nodes. Later slices cannot replace them with untyped integers or manufacture proof facts.
- Generated functions and callable members carry a private exact contract
  identity in addition to their prototype. These are body-proof obligations,
  not trusted effect flags; known contracts are catalogue-owned at stage 03.
- Extend the existing CObjectType foundation with the closed nominal categories. Separate known-library origins from generated origins; reject crossed registry/kind/owner references.
- Keep canonical identity/name ordering independent of transient allocation counters. No public source/certificate constructor or string-based symbol lookup.
- Typedefs cannot hide array parameter/return categories or effective const
  qualification. Derive expanded shape from actual registered targets and test
  nested aliases; do not weaken the existing private signature wrappers.

## Tests and proof

- Compile-fail controls for fabricated/cross-kind references and private registry evidence.
- Positive registration/lookup round trips; rejected cross-registry, wrong member owner, duplicate definition and alias-cycle controls; deterministic registration inventory.
- Existing type/declarator tests, Rustfmt/Clippy/Buildifier, full tracked/release/eight-target gates in Linux.

## Commit gate

Record exact commands, invocation IDs and outcomes. Commit and push this slice
with M34A-11-02A; keep its parent M34A-11-02 and overall C compliance open
until their remaining obligations pass. Use focused modules below the source
size limits and distinct Bazel targets only at real independent boundaries.
