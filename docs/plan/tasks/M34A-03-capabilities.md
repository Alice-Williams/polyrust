# M34A-03 — Add exhaustive capability strategies

- Status: planned
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
