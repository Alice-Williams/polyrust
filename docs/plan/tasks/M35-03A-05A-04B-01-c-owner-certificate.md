# M35-03A-05A-04B-01 — Strict C type-owner certificate

- Status: planned
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
