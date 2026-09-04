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
- Mapping inputs are portable typed AST or verified CoreIR values, never
  already-complete target AST passed through unchanged.
- Deferred capabilities are absent rather than represented by placeholders.

## Tests

- Compile-pass requirement inference for every initial capability.
- Compile-fail missing, duplicate, wrong-capability, wrong-dialect, and
  manually forged support.
- Compile-fail enum payload construction and incomplete interface bindings.
- Catalogue/layout policy proves one file per capability and exhaustive
  registration.
- Rustfmt, strict Clippy, Buildifier, documentation, full tracked repository,
  and release gates pass in the Linux development container.

## Commit gate

Commit and push the normative specification before implementation. Commit and
push the shared implementation only after all named tests pass.
