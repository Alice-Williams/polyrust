# C17 binary64 remainder mapping

- Status: target foundation implemented and verified; Rust-source admission pending
- Parent: [shared remainder](../../rust-floating-remainder.md)

## Typed mapping

Use CKnownCall::FloatRemainder, whose exact signature is double(double,double).
Do not use CBinaryOperator::Remainder on floating operands, IEEE remainder(),
or arithmetic expansion. Typed catalogue dependency traversal derives math.h
and CSystemLibrary::Math; consumers use the certified transitive manifest
inventory. All argument evaluation is materialized in Rust order before fmod.

## Certification and execution

Admit this closed call only after target, native and resource evidence. Keep
every existing known-call signature, ownership, recursive numeric and imported
authority check. Supply a conservative supported-profile stack reserve backed
by guarded-stack and watermark tests of real producer/importer calls. Do not
reuse trunc's measurements as evidence for fmod.

The pinned Linux binary64/nontrapping environment is required. Verify GCC14
and Zig O0/O2 against exact oracle expectations, including domain/nonfinite
categories. errno and floating flags are not observed by this profile.
[N1570](https://www.open-std.org/jtc1/sc22/wg14/www/docs/n1570.pdf), 7.12.10.1
and F.10.7.1, specifies fmod and the IEC 60559 cases; arbitrary ISO C library
implementations are not covered merely because their function is named fmod.

The certified profile now admits the exact catalogue call and reserves 64 KiB
for its supported library stack usage. Recursive admission and source/resource
accounting retain both operands and original dependency authority. The native,
mutation, guarded-stack and full-gate evidence is recorded in
[02O-02](../../../../plan/tasks/M35-03A-02O-02-c-remainder.md).
