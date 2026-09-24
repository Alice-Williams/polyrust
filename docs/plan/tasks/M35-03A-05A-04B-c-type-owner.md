# M35-03A-05A-04B — Certified C canonical type owner

- Status: planned
- Parent: [compiler results](M35-03A-05A-04-compiler-results.md)
- Depends on: [identity and placement](M35-03A-05A-04A-instance-ownership.md)
- Specification: [C canonical owners](../../specification/typed-generation/rust-canonical-type-owners.md#c17-specification)

## Contract

Add an explicit type-only owner profile for the existing certified scalar Result
layout, with version-2 error-kind semantics from 04A-03: its I32 member is active
under both tags, and Err stores the original checked kind code. Reuse typed
declarations, members, header imports and original dependency
authority. Do not forge a Rust source root/export inventory, broaden the ordinary
source-owner profile or add runtime code.

## Checkpoints

1. [04B-01 — strict descriptor and certificate](M35-03A-05A-04B-01-c-owner-certificate.md).
2. [04B-02 — typed dependency ownership](M35-03A-05A-04B-02-c-owner-imports.md).
3. [04B-03 — native and boundary proof](M35-03A-05A-04B-03-c-owner-proof.md).

Each receives an independent release/lint gate, review and commit. Compiler bundle
publication remains 04D; Result HIR source admission remains 04E.

## Implementation and definition of done

1. Store the typed owner descriptor in the immutable certificate. Derive opaque
   type/member handles only after validating exact selected layout and provenance.
2. Audit source-root-only assumptions in registries, closure checks, ABI manifests
   and source/stack bounds. Preserve all existing source-owned behavior.
3. Generate ordinary header/source packages with deterministic paths/guards;
   enforce actual identifier, file, owner and output-byte limits.
4. Compile independent producers against one original type owner, then a native
   cross-producer consumer with GCC and Zig. Cover success/every error kind, zero/extrema,
   original tag/member ownership and standalone headers.
5. Reject competing certificates, wrong profile/instance, forged source roots,
   missing imports, duplicated foreign declarations and unsupported helpers.
6. Prove zero executable frame contribution, exact/one-over resources, stable
   prior bytes, full Linux Bazel release/lint and independent GPT-6-SOL review.
   Commit/push separately; compiler Result source admission remains closed.
