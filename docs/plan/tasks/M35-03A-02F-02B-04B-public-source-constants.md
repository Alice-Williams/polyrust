# M35-03A-02F-02B-04B — Compiler public constant declarations and reads

- Status: in-progress
- Parent: [compiler public constants](M35-03A-02F-02B-04-source-constant-mappings.md)
- Depends on: M35-03A-02F-02B-04A

## Contract

Implement the shared public-constant source contract through executable
PublicConstants and PublicConstantReads bindings for C and Java. Resolve a closed
function/constant/module export inventory from compiler identities. Package
contexts register exact evaluated scalar constants before lowering function
bodies; body contexts read only the corresponding typed owned value references.

Preserve complete aliases, docs and effective visibility. Constants-only source
packages use the package-state path without a function Reader. Preserve existing
selected-entry semantics with an explicit documented selection policy; do not
silently substitute folded literals for authenticated public-package API reads.
Foreign producer joins and versioned multi-crate metadata remain child05 and must
fail closed until supported.

## Definition of done and tests

- Bool/i32/i64 declarations and reads work in real single-crate Rust/C/Java,
  including constants-only and mixed packages, computed values and forward reads.
- Compiler mapping probes verify actual typed declarations/references, exact
  identities, values, types, readonlyness, source ownership and emitted bytes.
- Missing, duplicate, wrong capability/context/input/output and forged compiler
  input controls exist for each new per-backend binding.
- Independent native consumers cover both bools, signed boundaries, wide values,
  aliases, private same-spelled constants and docs; native writes fail.
- Unsupported types, borrowed storage, generic owners and incomplete exports
  reject before publication. Compiling semantic mutants fail independent truth.
- Old public-constant rejection tests are replaced only alongside positive proof.
  All private/local constant regressions, full release/lint gates and independent
  review pass; ignored examples are exported and inspected before scoped push.

## Implementation sequence

1. [04B-01 — Shared typed export inventory](M35-03A-02F-02B-04B-01-export-inventory.md) — complete.
2. Register executable public constant declarations and reads for both targets,
   integrate owned single-crate metadata and constants-only publication, and add
   native/compiler/mutation proof required above.

The first checkpoint unifies source selection without enabling public constant
output. This parent stays in-progress until all declaration/read and publication
evidence passes; inventory classification alone is not capability support.
