# M35-03A-05A-04B-03 — C canonical-owner native proof

- Status: planned
- Parent: [C type owner](M35-03A-05A-04B-c-type-owner.md)
- Depends on: [typed imports](M35-03A-05A-04B-02-c-owner-imports.md)
- Specification: [canonical owners](../../specification/typed-generation/rust-canonical-type-owners.md#c17-specification)

## Definition of done and tests

- Two producers use the same original type-only certificate. Compile separate C
  objects and a cross-producer consumer with strict GCC and Zig; exercise both
  variants, zero, signed extrema, payload forwarding and original member identity.
- Compare an independent expected-value oracle; include compiling wrong-tag,
  wrong-payload and duplicate-evaluation controls where executable composition
  occurs. No target text mutation counts as a typed generation success.
- Permuted registration and unrelated consumers preserve original package bytes.
- Exact/one-over owner, descriptor, identifier, source and closure limits fail
  at the expected boundary; no shadowed owner or foreign redeclaration passes.
- Export actual generated examples without committing generated files.
- Preserve recorded old outputs/WIP, pass full Linux Bazel release/lint and
  independent GPT-6-SOL review, then scoped commit/push.

This completes the C target owner foundation only. Graph publication and Rust HIR
Result admission remain 04D and 04E; no legacy-removal approval is implied.
