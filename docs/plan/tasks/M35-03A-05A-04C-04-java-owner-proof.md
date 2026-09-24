# M35-03A-05A-04C-04 — Java canonical-owner native proof

- Status: planned
- Parent: [Java canonical owner](M35-03A-05A-04C-java-type-owner.md)
- Depends on: [typed imports](M35-03A-05A-04C-03-java-owner-imports.md)
- Specification: [canonical owners](../../specification/typed-generation/rust-canonical-type-owners.md#java21-specification)

## Definition of done and tests

- Generate two independently certified producers against one original type
  owner, plus generated consumers composing A-to-B and B-to-A.
- Compile owner/producers/consumers separately with strict Java21 diagnostics;
  run normal and interpreted modes with an independent value and call-trace
  oracle. Cover all six distinct error constants and successful zero, signed
  ranges and extrema, original success accessors and enum forwarding.
- Compiling typed wrong-kind, wrong-success-payload, duplicate and reordered
  controls must fail the intended oracle, not compilation or an incidental
  exception. Foreign null input still fails at the documented boundary.
- Check original authority and byte determinism under registration permutations
  and unrelated instances. Map actual exact/one-over owner, namespace, closure,
  source and classfile proofs to the unchanged resource policies.
- Export actual generated packages; preserve prior output and protected WIP
  hashes. Full Linux Bazel release/lint, independent GPT-6-SOL review and scoped
  commit/push close Java's type-owner foundation only.

Rust/C/Java checked-source comparison and atomic graph output follow in 04D/04E;
this fixture alone does not authorize source admission or legacy removal.
