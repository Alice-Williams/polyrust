# M34A-09 — Seal manifests and build the evidence harness

- Status: complete
- Depends on: M34A-08

## Goal

Make the generic compiler adapter the only path from checked input to a safe
artifact manifest and provide reusable proof for every language migration.

## Definition of done

- Only the shared assembler constructs `OutputManifest` from verified rendered
  files and declared non-source artifacts.
- Path safety, role/media type, size/count limits, duplicates, normalization,
  newline/encoding, executable-bit, and deterministic ordering are validated.
- The object-safe registry exposes the sealed compiler adapter without a direct
  plugin generation escape.
- A reusable compliance kit runs canonical semantics, interface/composition,
  dependency/helper minimality, renderer isolation, manifest security, and
  three-generation checks for any dialect.
- Fault injection proves unresolved AST, malformed render views, extra files,
  and plugin-created manifests cannot pass.
- The external-backend example is adapted to the new safe boundary or is
  explicitly scheduled for final cleanup without claiming compliance.

## Tests

- `bazel test //crates/codegen:manifest_v2_test //crates/codegen:plugin_adapter_test --nocache_test_results --test_output=errors`
- `bazel test //crates/codegen:typed_backend_contract_test --nocache_test_results --test_output=errors`
- `bazel test //examples/external-backend:all --nocache_test_results --test_output=errors`
- Path/security/limit/atomicity, phase-bypass compile-fail, and deterministic
  manifest fault-injection tests.

## Commit gate

Commit and push `M34A-09: seal typed generation manifests` only after all
focused shared-platform and external-extension tests pass in the dev container.

## Exit evidence

- Manifest v2 records target/backend/IR identity, typed generation options,
  file role and media type, a stable content hash, executable-bit policy,
  exact dependency feature sets, and exact helper capability/file reports.
  Only the shared compiler adapter can call its constructor.
- `RenderedPackage`, `RenderedFile`, verified/resolved phase wrappers, and the
  sealed `TargetRenderer` bridge have inaccessible constructors or sealed
  implementation traits. Compile-fail tests prove an external plugin cannot
  manufacture a rendered package, implement a bypass renderer, render
  unresolved AST, recover unresolved state from a linked package, or construct
  an `OutputManifest`.
- The certified renderer revalidates the linked package, renders typed source
  views, copies only verified documentation and binary artifacts, derives
  sorted dependency features and helper reports from typed dialect enums, and
  fails explicitly for metadata or derived JavaScript until their required
  language-specific renderer/compiler phases migrate.
- Manifest verification rejects unsafe/reserved/colliding paths, duplicate or
  unordered files/dependencies/features/helpers, role/media mismatches,
  executable outputs, forged hashes, missing helper files, CR/NUL text,
  incorrect final newlines, excessive file counts, and per-file/package size
  violations before any CLI materialization.
- `prove_typed_compiler` runs the sealed object-safe compiler three times and
  records stable target/hash/semantic/interface evidence. Its oracle verifies
  canonical semantics and interface/composition behavior while exact expected
  dependency/helper sets prove minimality; negative fixtures alter each class
  of expectation and are rejected.
- The old `Backend`/language-document route is explicitly isolated under the
  hidden `legacy` module. `examples/external-backend` remains a compatibility
  test only and is explicitly scheduled for deletion/adaptation in M34A-18; it
  is not claimed as typed-compliant here.
- In the Linux development container, all named M34A-09 gates passed uncached.
  Rustfmt, Clippy, Buildifier, dependency-boundary, template-policy, and typed
  source-policy gates passed. The complete tracked-scope graph passed uncached,
  266 of 266 tests. The frozen untracked M34-03 `stdlib-abs` package was the
  only excluded Bazel package and was not modified.
