# M34A-02 — Introduce canonical verified CoreIR

- Status: complete
- Depends on: M34A-01

## Goal

Give every target one normalized, typed, target-neutral semantic input.

## Definition of done

- A dedicated CoreIR module/crate implements all types, declarations,
  expressions, statements, operations, interfaces, implementations, portable
  tests, source provenance, and typed arena IDs required by current v0.
- Closed behavior choices are exhaustive enums; input-defined declarations use
  typed IDs and no code compares ID text to select semantics.
- Lowering from `CheckedProgram` removes authoring sugar and makes
  receiver/argument evaluation order and non-duplicable temporaries explicit.
- The independent verifier checks types, references, callable signatures,
  control flow, exhaustiveness, interface conformance, evaluation order, and
  absence of target concepts.
- The reference evaluator consumes or is proven equivalent to verified CoreIR.
- Canonical dumps are stable and every existing checked fixture lowers.
- Invalid/fabricated CoreIR can be exercised only by test support and is
  rejected deterministically.

## Tests

- `bazel test //crates/core-ir:all --nocache_test_results --test_output=errors`
- `bazel test //crates/eval:core_ir_parity_test --nocache_test_results --test_output=errors`
- Every-node, invalid-reference/type/order, no-target-data, canonicalization,
  and three-lowering determinism fixtures.

## Commit gate

Commit and push `M34A-02: add verified canonical CoreIR` only after focused
tests plus checker/evaluator suites pass in the dev container.

## Evidence

- `portable_core_ir` defines canonical typed arenas for all v0 declarations,
  types, values, constants, expressions, blocks, interfaces,
  implementations, tests, source provenance, evaluation order, and immutable
  result ownership. Closed operations and intrinsic arities are enums;
  declarations and members use category-specific IDs.
- `CanonicalCoreLowerer` exhaustively maps checked v0 into canonical CoreIR,
  resolves aliases, separates static/interface dispatch, preserves
  left-to-right postorder, and validates the result before returning it.
- The independent verifier derives expression/intrinsic types, validates every
  reference and owner, exact callable and interface conformance, aggregate
  fields, match coverage, block results, lexical dominance, return types,
  portable tests, provenance, and canonical type uniqueness. Production
  `CoreProgram` construction remains private; crate-only mutation hooks support
  deterministic reference/type/order fault injection.
- The focused fixture covers constants, records, interfaces, explicit
  conformance, static and dynamic dispatch, `Option` matching, conditionals,
  bounded iteration, tests, no target concepts, and three-pass byte-identical
  canonical dumps.
- `//crates/eval:core_ir_parity_test` replays the interface model and all 12
  completed real-world ports. Their reference-evaluator vectors pass, CoreIR
  verifies, three lowerings are byte-identical, test identities are unchanged,
  and normalized typed values are identical after source-ID to Core-ID
  remapping.
- Linux-container Bazel invocation
  `9e7b55bd-2768-4a86-8b0b-ae9587c221a9` passed the CoreIR, parity, checker,
  evaluator, Buildifier, Rustfmt, and Clippy gates.
