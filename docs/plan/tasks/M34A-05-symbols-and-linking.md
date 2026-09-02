# M34A-05 — Implement typed symbols and linking

- Status: planned
- Depends on: M34A-04

## Goal

Derive every target dependency, helper edge, name, and reference spelling from
typed AST references rather than manually attached strings.

## Definition of done

- Stable typed IDs distinguish generated, known external, runtime-helper,
  local, package, file, and test symbols.
- Known types, callables, fields, constructors, methods, and invocation kinds
  have authoritative typed catalogue entries with owner/origin, full
  signatures, generics, failure/effect metadata, and dependency policy.
- Known-call constructors enforce precise receiver/argument/result types at
  Rust compile time where types are closed; generated calls are verifier
  checked.
- The shared resolver traversal collects references once, expands helper
  closure, detects missing/cyclic symbols, allocates collision-safe names, and
  produces an opaque `ResolvedPackage<D>`.
- Imports/includes/qualification, package dependencies, and forward
  declarations are language resolver outputs; lowerers cannot add them.
- Positive/negative dependency and collision matrices use the test dialect.
- No behavior is selected by comparing symbol/ID spelling.

## Tests

- `bazel test //crates/codegen:symbol_catalogue_test --nocache_test_results --test_output=errors`
- `bazel test //crates/codegen:target_linker_test --nocache_test_results --test_output=errors`
- `bazel test //crates/codegen:typed_call_compile_fail_test --nocache_test_results --test_output=errors`
- Exact dependency presence/absence, alias/qualification, missing/cycle,
  generated-signature, and three-resolution determinism fixtures.

## Commit gate

Commit and push `M34A-05: derive dependencies from typed symbols` only after
focused and shared-codegen tests pass in the dev container.
