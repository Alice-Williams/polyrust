# M34A-09 — Seal manifests and build the evidence harness

- Status: planned
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
