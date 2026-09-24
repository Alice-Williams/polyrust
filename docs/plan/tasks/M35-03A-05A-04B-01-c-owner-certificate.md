# M35-03A-05A-04B-01 — Strict C type-owner certificate

- Status: complete
- Parent: [C type owner](M35-03A-05A-04B-c-type-owner.md)
- Depends on: [checked instances](M35-03A-05A-04A-instance-ownership.md)
- Specification: [canonical owners](../../specification/typed-generation/rust-canonical-type-owners.md#c17-specification)

## Contract and implementation

Add a mutually exclusive canonical-type registry profile with complete original
facts, closed C representation version and exact header binding. The frozen
registry retains it before projection and independent certificate reconstruction.
Do not fabricate RustSource provenance, core exports or source-owned functions.

The profile accepts one exact public Bool/I32 scalar-result struct and its two
registered members, a deterministic header/source pair and required platform
assertions. No executable/global/helper inventory. The source companion derives
its own header dependency structurally. Keep ordinary source profile checks intact.

Use the closed version-2 descriptor: the I32 field represents the success value
or original error-kind code according to the tag. This descriptor cannot claim
compiler-state authentication; 04A-03 supplies that distinct source witness.

## Definition of done and tests

- Worst-case naming/header guards and exact file/role/layout/profile are checked.
- Native standalone header/source compile with strict GCC and Zig.
- Measured executable frame contribution is zero, not a caller assertion.
- Wrong profile/facts/names/files/members, extra declarations, forged core source
  exports and conflicting source/type registration reject before certification.
- No imports are claimed yet: the existing source-only dependency API must still
  reject type-only packages until 04B-02 explicitly handles their certificate.
- Unchanged scalar output, full Linux Bazel release/lint, independent GPT-6-SOL
  review and scoped commit/push precede completion.

## Implementation

The registry uses one private enum for mutually exclusive source and canonical
ownership. The frozen type descriptor retains all fifteen descriptive original
identities, version 2 and the original registered file/record/member handles.
It cannot authenticate compiler provenance or manufacture a render certificate.

The strict profile reconstructs exact names, member order/types and declaration
inventory. Platform assertions have canonical messages/order as well as checked
conditions, so caller formatting cannot change an owner's bytes. The ordinary
source rules remain closed to empty executable inventories; only this exact
profile admits one. Resource certification measures its actual zero executable
contribution. Source-only dependency publication explicitly rejects it for now.

## Evidence

The six focused Linux Bazel targets pass on code tree
`26dfe003d250e3cb58c34558b8f9a6775a0523e2`; invocation
`36500abf-d74c-4bca-84e7-c8fdbe3d2f61`, 155.852 seconds. This includes:

- Seven dedicated certificate/native cases: malformed names/files/member shapes,
  extra registrations/content, fake core provenance, changed facts, source/type
  conflicts, cross-registry roles, deterministic output and assertion mutations.
- GCC and Zig standalone compilation/link/execution at O0/O2 with ordinary and
  maximum-width identity names. Duplicate header inclusion succeeds and native
  symbol inspection confirms the generated implementation exports no executable
  symbol. No runtime file or source crate is needed.
- All 886 selected C unit cases, zero failures/ignored cases; eleven unchanged
  native partitions retain their separate Bazel targets. Actual linked-package
  measurement proves zero total/per-file executable frames and nonzero source
  bounds. Even an unused, genuinely certified foreign struct is rejected.
- Existing compile-fail checks plus Rustfmt, Clippy and Buildifier. Initial
  Clippy findings were three needless borrows in test paths, now corrected.

- Two independent GPT-6-SOL exact-tree reviews are clean. They cover certificate
  reconstruction, owner separation, canonical assertions, structural imports,
  measured storage and the new native/negative tests. No core finding remains.

Full Linux Bazel release/lint passes all 1,052 targets, with 131 executed and
unchanged results cached: invocation `048347ea-fe31-47d9-9478-b1106b700297`,
1,352.144 seconds. All 530 recorded generated files remain byte-identical and
all 45 protected WIP files remain untouched and excluded from this checkpoint.
Prepared 04B-02 tests/planning are excluded too.

Dependency imports and Rust Result source admission remain closed; full six-state
target transport is 04B-03. This certificate describes valid target syntax and
placement, not independent compiler authentication of its original Rust facts.
