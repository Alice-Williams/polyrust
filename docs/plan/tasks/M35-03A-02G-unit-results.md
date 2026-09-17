# M35-03A-02G — Unit function results and effect-only calls

- Status: complete
- Parent: [scalar parity](M35-03A-02-scalar-parity.md)
- Depends on: M35-03A-02F
- Specifications: [shared](../../specification/typed-generation/rust-unit-results.md),
  [C](../../specification/typed-generation/languages/c/rust-unit-results.md),
  [Java](../../specification/typed-generation/languages/java/rust-unit-results.md)

## Contract

Add built-in Rust () function results through the existing compiler -> typed
target AST -> certificate -> rendering pipeline. Use ordinary C/Java void.
Do not add unit storage/parameters, tuples, null substitutes, runtime helpers,
I/O, panic or arbitrary effects. Retain exact scalar argument order and callable
authority; unit is an effect-only result, not a scalar value.

## Ordered checkpoints

1. [02G-01 — Certified target void results](M35-03A-02G-01-target-void.md):
   complete: shared C primitive-result vocabulary, bounded C/Java source/dependency
   profiles, ordinary typed void statements/returns and native target proof.
   Exact-tree release gate: 742/742 tests; fresh independent review: no findings.
2. [02G-02 — Compiler unit effects and publication](M35-03A-02G-02-compiler-unit-effects.md): checked UnitInput and real
   executable mappings/builder slots, signature/body/call integration, versioned
   metadata and real multi-crate native/source-order proof. Its bounded
   task is complete: all 760 release tests pass; a fresh independent Sol Extra
   High review found no findings. Actual generated packages are exported outside
   Docker under ignored generated/examples/unit-results-597b335/.

## Definition of done

- Every admitted unit source shape has an executable mapping, private checked
  source input and typed effect output; no unsupported form silently disappears.
- Local/private/public/transitive functions preserve void ABI and scalar argument
  evaluation, branch selection, declaration identity, docs and producer closure.
- Exact versioned manifests, AST evidence, compilation-negative cases,
  authoritative dependency tamper tests and atomic publication controls pass.
- Rust/C/Java native and sequencing proof, full Linux Bazel/release/lint gates,
  exported examples and fresh independent Sol Extra High review pass.
- Each checkpoint has its own evidence, scoped commit and push. General unit
  values/storage and the overall runtime migration remain incomplete.
