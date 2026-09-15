# M35-02B-03K — Authenticate scalar Box cloning

- Status: in-progress
- Parent: [M35-02B-03](M35-02B-03-structured-owned-places.md)
- Depends on: M35-02B-03J
- Specification: [scalar Box clone](../../specification/typed-generation/languages/c/rust-owned-clone.md)

## Contract

Prove the remaining standard clone obligation before C mapping. Cloning a scalar
Box borrows the source and creates a distinct owner; it is not a move, an alias,
or a local function whose signature alone establishes effects. Keep canonical
HIR as the structured input and actual compiler identities/types as authority.

## Ordered implementation

1. [K-01 — Clone identities](M35-02B-03K-01-clone-identities.md): observe the
   pinned compiler, authenticate a closed standard operation and provide its
   executable typed capability registration. No whole-body admission.
2. [K-02 — Clone correspondence](M35-02B-03K-02-clone-correspondence.md): prove
   a bounded source/clone/read/cleanup body with independent original and cloned
   owner chains, canonical exits and complete MIR accounting.

## Definition of done and tests

- Each child has a clean independent review and exact-tree full Linux/Bazel gate.
- Same-named functions, cloning a reference, wrong payloads and substitutions
  cannot acquire the scalar Box clone capability.
- Missing, repeated or swapped owner cleanup and borrowed-source substitutions
  fail the correspondence oracle. Native allocator proof remains M35-02D.
- No target heap support is announced or enabled by compiler-only evidence.
