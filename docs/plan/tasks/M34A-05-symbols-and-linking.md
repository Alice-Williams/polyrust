# M34A-05 — Implement typed symbols and linking

- Status: complete
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

## Evidence

- Category-specific IDs and closed reference/origin enums distinguish generated
  types/callables/interface methods/values, known types/callables/fields/
  constructors/methods, runtime callables/helpers, locals, type parameters,
  packages, files, and tests. Resolution never branches on an ID or symbol's
  textual spelling.
- `SymbolCatalogue<D>` records typed ownership, identifiers, qualified/member
  names, invocation, generic type patterns, receiver/parameter/result types,
  visibility, failure behavior, effects, origin, import policy, and exact
  external package requirements. Validation rejects duplicate entries,
  malformed generic references, inconsistent concrete signatures, invalid
  qualification, and origin/dependency mismatches.
- Phantom-typed builders cover static nullary/unary/binary calls, nullary
  constructors, and unary instance calls with distinct receiver, argument, and
  result markers. Compile-fail cases prove wrong static arguments and wrong
  instance receivers do not compile; generated calls retain verifier-checked
  dynamic signatures.
- `TargetLinker<D>` performs the authoritative file-root AST traversal, helper
  closure, collision-safe binding allocation, typed reference resolution,
  import/alias/qualification/member selection, external dependency unification,
  and forward-declaration derivation. Its only public direct entry verifies the
  unresolved AST first; the phase adapter consumes `UnresolvedPackage<D>`.
- The resolved verifier rejects forged bindings, references, imports,
  dependencies, helpers, files, and forward declarations. The test dialect
  proves import presence/absence, prelude and qualified paths, case-insensitive
  collisions, stable private renames, public collision failure, nested/unused/
  missing/cyclic helpers, version conflicts, missing symbols, and three
  byte-identical non-source link dumps.
- The typed-generation source policy now covers both production AST and linker
  sources and fault-injects opaque executable nodes, document/string escape
  hatches, manual dependency attachment, and import/include text scanning.
- Linux-container Bazel invocation
  `a616c927-bf56-4f58-96a5-0792f7eb8219` passed every codegen unit/doctest,
  the typed-generation policy, Buildifier, Rustfmt, and Clippy: 15/15 tests.
