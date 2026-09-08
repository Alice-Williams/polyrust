# M34A-11-02A — C registries and nominal identities

- Status: planned
- Depends on: M34A-11-00R, M34A-11-01R, M34A-10AB

## Goal

Implement this bounded part of M34A-11-02 without introducing a raw-source path
or advertising capabilities before their mappings exist.

## Definition of done

- Add private registry-scoped, kind-specific identities for struct, union, typedef, enum, function, object, member, parameter, local and file registrations. References retain origin, owner and complete structural type/signature.
- Extend the existing CObjectType foundation with the closed nominal categories. Separate known-library origins from generated origins; reject crossed registry/kind/owner references.
- Keep canonical identity/name ordering independent of transient allocation counters. No public source/certificate constructor or string-based symbol lookup.

## Tests and proof

- Compile-fail controls for fabricated/cross-kind references and private registry evidence.
- Positive registration/lookup round trips; rejected cross-registry, wrong member owner, duplicate definition and alias-cycle controls; deterministic registration inventory.
- Existing type/declarator tests, Rustfmt/Clippy/Buildifier, full tracked/release/eight-target gates in Linux.

## Commit gate

Record exact commands, invocation IDs and outcomes. Commit and push this slice
with M34A-11-02A; keep its parent M34A-11-02 and overall C compliance open
until their remaining obligations pass. Use focused modules below the source
size limits and distinct Bazel targets only at real independent boundaries.
