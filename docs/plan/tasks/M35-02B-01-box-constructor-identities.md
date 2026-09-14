# M35-02B-01 — Authenticate Box construction capability inputs

- Status: complete
- Parent: [M35-02B](M35-02B-structured-owned-mapping.md)
- Depends on: M35-02A

## Contract

Reuse the existing executable Capability/Mapping/Supports contracts; do not add
an independent support flag or duplicate trait hierarchy. A private typed
constructor input retains compiler HIR expression/body identity, actual FnDef,
instantiated arguments and Box/result types. Recognize standard construction
through `box_new` diagnostic identity and `owned_box` type identity, not spelling.

The first admitted operation is `Box::new(i32)`. This does not admit a whole
program, select an allocator for C, authenticate HIR/MIR variable correspondence,
or enable backend heap translation. Match constructor inventories between HIR
and drop-elaborated MIR as observation only, without assigning variable/exit
identity from equal counts.

## Definition of done and tests

- Genuine, fully qualified and renamed-import calls share the compiler identity.
- Same-named local constructors and functions returning a real Box do not gain
  standard-constructor authority. Unsupported payload types reject explicitly.
- Input validation checks canonical HIR node/body ownership and concrete
  instantiated signature/result agreement. Mismatched owner/node inputs reject.
- A consuming single-slot builder registers an executable mapping through the
  shared Supports contract; the probe invokes that function with the typed input.
- Compile-negative tests reject missing/duplicate registrations and wrong
  capability/input/output/context, without suppressing unrelated diagnostics.
- Pinned Clippy, Rustfmt, focused compiler tests, full isolated C/Java/release
  gates and fresh review pass. Commit/push only the verified checkpoint tree.

## Proof limit

No public target API is changed. Successful rustc analysis remains mandatory in
the compiler callback. Compiler declaration identity plus type agreement is
constructor evidence, not evidence of target cleanup or ownership correspondence.

## Verified checkpoint

- Focused gate `8f639192-5778-4f00-b0e7-2691e0ec7b0f`: 11/11 tests passed,
  including seven compile-negative targets and fixture/input mutations.
- Isolated tree `1a777d41d29379cb5df984866edf6a5c6d8c6970`: complete gate
  `6d2e46ad-b059-4f3f-a818-8bc1723a9d22` passed 384/384 tests across 500 targets
  in 150.862 seconds. Forty tests executed; valid cached results were retained.
- Gate includes release/native language proofs, all compiler-experiment targets,
  C/Java/shared codegen and negative contracts, Rust/Bazel lint/format and docs.
  All historical tests remained enabled; unrelated M34 work stayed excluded.
- Fresh independent Sol Extra High review found no core findings. The review
  checked actual constructor authority, canonical input privacy, executable
  binding/factory constraints, inventory scope, mutations and test dependencies.
- Git archive bytes and executable modes were checked against the exact tree.
  The final documentation-inclusive tree and gate are recorded in the checkpoint
  commit message after successful verification, before push.
