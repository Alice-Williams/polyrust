# M34A-08V — Make capabilities the typed portable extension unit

- Status: complete
- Depends on: M34A-08U
- Blocks: M34A-10W and every remaining language migration

## Goal

Replace the transitional feature terminology and monolithic catalogue with a
closed semantic capability system whose definitions have one source file per
capability.

## Definition of done

- The public shared traits are `Capability`, `CapabilityMapping`,
  `Supports<C>`, and `SupportsAll<R>`; no parallel static Feature vocabulary
  remains.
- `crates/build/src/capabilities/` contains exactly one file per initial
  capability plus a machinery-only `mod.rs`.
- Functions own calls/returns, records own construction/projection,
  interfaces own all polymorphic forms, and enums are payload-free.
- Typed constructors infer catalogue capabilities and callers never supply
  support evidence.
- `typed_program` itself infers `Modules`; declaration builders cover
  constants, aliases, functions, records, payload-free enums, interfaces,
  implementations, and portable tests.
- Typed blocks and statements cover immutable locals, conditionals, loops,
  non-enum pattern matching, and explicit result propagation. Loop controls,
  locals, fields, enum variants, interface methods, and implementation
  bindings are branded so they cannot be used outside their owner. The v0
  `Loops` surface is bounded `for_each`; unbounded loops and loop-control
  statements remain deferred semantics.
- Mapping inputs are portable typed AST or verified CoreIR values, never
  already-complete target AST passed through unchanged.
- Deferred capabilities are absent rather than represented by placeholders.

## Tests

- Compile-pass requirement inference for every one of the 42 initial
  capabilities, with an inventory test naming the constructor which owns each
  capability leaf.
- Compile-pass arbitrary-length function, record, enum, interface-method, and
  implementation-binding lists with at least three elements each.
- Compile-fail missing, duplicate, wrong-capability, wrong-dialect, and
  manually forged support.
- Compile-fail enum payload construction and incomplete interface bindings.
- Compile-fail wrong constant/reference type, wrong alias use, cross-body
  local, escaped loop control, non-exhaustive option/result/bool match, and
  incompatible result propagation.
- Catalogue/layout policy proves one file per capability and exhaustive
  registration.
- Rustfmt, strict Clippy, Buildifier, documentation, full tracked repository,
  and release gates pass in the Linux development container.

## Commit gate

Commit and push the normative specification before implementation. Commit and
push the shared implementation only after all named tests pass.

## Completion evidence

- `crates/build/src/capabilities/` contains 42 marker files; the layout policy
  proves an exact one-file, one-export, one-catalogue-row relationship.
- The typed frontend covers all declaration, value, operation, control-flow,
  payload-free enum, interface/conformance, and portable-test constructors.
  `ContainsCapability<C>` folds inferred requirement trees into the closed
  catalogue, and compiled assertions cover every inventory row.
- Arbitrary three-element function, record, enum, interface-method, and exact
  implementation-binding lists pass. Incomplete/cross-branded interfaces,
  wrong test arguments/results, absent capability claims, and the other named
  invalid programs fail Rust compilation.
- Typed portable tests pass in the reference evaluator for normal function
  results, structured checked-overflow errors, and concrete implementation
  method invocation.
- Linux dev-container `bazelisk --batch test //:release_gate
  --test_output=errors`: 238/238 passed on 2026-09-05.
- Linux dev-container full tracked graph, excluding only the unrelated
  untracked partial `examples/real-world/stdlib-abs/` package:
  `bazelisk --batch test --test_output=errors -- //...
  -//examples/real-world/stdlib-abs/...`: 300/300 passed on 2026-09-05.
