# M34A-08V — Make capabilities the typed portable extension unit

- Status: in-progress
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
  bindings are branded so they cannot be used outside their owner.
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
