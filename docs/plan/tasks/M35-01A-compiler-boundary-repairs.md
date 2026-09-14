# M35-01A — Close compiler prototype review gaps

- Status: complete (local evidence; push held for the C/Java migration gate)
- Depends on: the existing M35-01 prototype
- Blocks: completion of [M35-01](M35-01-rustc-adapter.md)
- Contract: [C HIR lowering](../../specification/typed-generation/languages/c/rust-hir-lowering.md)

## Goal

Resolve the demonstrated independent-review findings before the prototype
is called complete. These are contract repairs, not new feature requests.

## Definition of done

- Require safe, non-variadic Rust-ABI entry metadata, not just parameter/result types.
- Preserve nested source blocks with structural block nodes and C braces.
- Sanitize ambient RUSTC_BOOTSTRAP for customer input compilation.
- Evaluate and record each review finding. Do not label previous positive
  parity tests as evidence for these previously untested cases.

## Tests and proof

- Valid extern-C entry fails admission and leaves output absent/unchanged.
- A nested tail block retains its scope/braces and matches native Rust.
- Feature-gated source rejects both ordinarily and under RUSTC_BOOTSTRAP=1.
- Existing compiler-negative and both 8,204-input O0/O2 parity suites pass.
- Rustfmt, Clippy, Buildifier and documentation gates pass in Linux/Bazel.
- Independent follow-up review finds no unresolved demonstrated core defects.

## Commit gate

Linux/Bazel invocation `37fb64ac-0f9d-4ae8-8e5c-0bc2a65713bb` passed all eight
checks: three native parity suites, compiler boundary negatives, Rustfmt,
reference Clippy, Buildifier and documentation. Each parity suite compares
8,204 inputs against Rust and C at O0/O2. Independent Sol Extra High follow-up
review passed after replaying the repaired boundaries. The README command now
includes the scope and boundary regressions.

Keep prior unfinished backend-c work separate. The user's latest instruction
holds all pushes until the C/Java migration tests are green; do not disable
regressions to satisfy this gate.
