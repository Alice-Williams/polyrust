# M34A-02 — Introduce canonical verified CoreIR

- Status: planned
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
