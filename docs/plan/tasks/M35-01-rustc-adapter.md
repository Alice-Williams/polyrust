# M35-01 — Compiler adapter and native proof

- Status: complete (local experiment; not production integration)
- Depends on: M00

## Goal

Establish the compiler-to-C handoff with real Rust source and compiler-owned
parsing, typing and borrow checking.

## Definition of done

- Pin compiler-development components in Bazel with verified downloads.
- Isolate unstable compiler APIs from production crates and toolchain defaults.
- Complete Rust analysis before extracting HIR plus compiler type-check
  results; use closed admission matches for types, places and expressions.
- Preserve structured branches and scopes. The output tree has no goto or
  label node; no control-flow reconstruction belongs in the renderer.
- Generate structural C deterministically; invalid and unsupported inputs
  fail before creating output.
- Compare native Rust and C on boundary and ordinary input vectors.
- Document commands, generated artifact paths and exact limitations.

## Tests and proof

- Positive non-Copy struct move, shared borrow, field access and branch cases.
- Compiler-negative use-after-move, escaping borrow and wrong-type cases.
- Valid unsupported Rust gets a distinct admission diagnostic.
- No artifact on either failure; repeated generation is byte-identical.
- Generated branches contain if/else and no goto statements or labels.
- Strict C17 native comparison at O0/O2, Rustfmt, Clippy, Buildifier and docs.
- Fresh independent review; evaluate every demonstrated core error.

## Scope boundary

No heap allocation, custom Drop, arbitrary library calls, unsafe Rust, FFI,
async or general trait-object lowering. This is an isolated experiment, not a
production C capability or memory-safety certificate.

## Commit gate

Independent review identified three contract gaps: entry ABI, nested scopes
and ambient RUSTC_BOOTSTRAP. [M35-01A](M35-01A-compiler-boundary-repairs.md)
records their repairs, dedicated regressions and passing follow-up review.
Linux/Bazel invocation `37fb64ac-0f9d-4ae8-8e5c-0bc2a65713bb` passed eight
checks, including three programs over 8,204 inputs each against C17 O0/O2.

The typed C bridge and Java retrofit remain unfinished. Per the latest user
instruction, hold pushes until the complete migration gate is green.
