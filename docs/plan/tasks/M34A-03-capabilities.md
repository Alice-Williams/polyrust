# M34A-03 — Add exhaustive capability strategies

- Status: complete
- Depends on: M34A-02

## Goal

Make every portable feature use and every target support decision typed,
complete, and independently testable.

## Definition of done

- `CoreFeature` is a closed enum hierarchy covering every current type,
  declaration, operation, control, interface, and ownership feature.
- CoreIR feature collection is structural, deterministic, and source-located.
- `FeatureSupport<S>` has explicit native, emulated, and unsupported variants
  with typed strategy/reason data.
- Every built-in plugin registry exhaustively acknowledges every feature with
  no wildcard/default/string-key path.
- A missing feature rejects only a requested program-target pair which uses it.
- The all-eight compatibility profile requires all eight strategies to be
  native or emulated.
- Backend options are typed and validated before preflight.
- Compile-time exhaustiveness and runtime positive/negative matrices become
  permanent gates.

## Tests

- `bazel test //crates/codegen:capability_registry_test --nocache_test_results --test_output=errors`
- `bazel test //crates/codegen:capability_compile_fail_test --nocache_test_results --test_output=errors`
- Feature-use minimality, unsupported isolation, all-eight rejection, option
  validation, stable diagnostic, and deterministic ordering tests.

## Commit gate

Commit and push `M34A-03: make capabilities exhaustive` only after focused
tests and all backend capability tests pass in the dev container.

## Evidence

- `CoreFeature` is a closed family over declarations, types, controls,
  interfaces, operations, and ownership. Intrinsic operation features embed
  the exhaustive CoreIR unary/binary/ternary/variadic enums rather than names.
- Structural collection produces canonical, deduplicated `FeatureUse` values
  with typed aggregate/callable/interface/variadic shape and nearest source
  provenance. Tests prove minimality, stability, and absence of unused string
  features.
- `SupportDecision<S>` separates native, emulated, and closed-reason
  unsupported decisions. Preflight validates that every selected strategy has
  a registered lowering and returns all stable target/source diagnostics.
- Eight compile-time registries explicitly match all feature families and
  nested intrinsic variants. JavaScript selects compiler-derived TypeScript;
  C interface values/dynamic dispatch select function-table emulation.
- Typed option proof objects are private to successful validation. A
  panic-on-query registry proves invalid options stop before capability
  preflight. Matrix tests prove target isolation and atomic all-target failure.
- CoreIR no longer depends on codegen. Codegen owns `CanonicalCoreAdapter` and
  depends one-way on CoreIR, allowing capability preflight without a crate
  cycle.
- Linux-container Bazel invocation
  `465786e7-0c2b-4fda-8ab2-99ce7755fb4c` passed codegen, all seven independent
  backend regression targets (JavaScript remains TypeScript-derived),
  Buildifier, Rustfmt, and Clippy: 16/16 tests.
