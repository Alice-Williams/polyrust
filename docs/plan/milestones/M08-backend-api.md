# M08 — Implement backend API, registry, preflight, and manifests

- Status: planned
- Phase: 2
- Depends on: M02, M03, M04, M07

## Outcome

Define the extension boundary used equally by Rust, TypeScript, Python, Go, and
future external backends.

## Implementation checklist

- `Backend` trait, descriptor, target ID, options schema, support level, and
  version compatibility types.
- Runtime registry owned by the CLI/application layer.
- Whole-program capability preflight with node-level diagnostic traces.
- `OutputManifest`, files, dependencies, and injected-helper metadata.
- Manifest validation for path traversal, absolute/drive paths, duplicates,
  case-fold collisions, invalid text, and deterministic ordering.
- In-tree mock backend plus contract-test kit reusable by external backends.

## Required exit evidence

- Target ID is open namespaced text, not a closed enum.
- Core crates have no concrete target-name branches.
- Preflight reports all unsupported capabilities before calling `generate`.
- Backends accept `CheckedProgram`, never unchecked IR.
- Manifest validation is complete before filesystem APIs are invoked.
- A backend's reported descriptor and capability table are deterministic.

### Verification

- Mock backend success, unsupported capability, invalid option, incompatible IR,
  and deliberate backend-error tests.
- Path corpus covering `..`, rooted paths, Windows drive/UNC forms, separators,
  Unicode/case collisions, duplicate files, and reserved metadata paths.
- Registry duplicate-ID and lookup tests.
- Test proving a new mock target registers without modifying core source.
- Manifest serialization/determinism property tests.

```text
cargo test -p polyrust-codegen
```

### Completion gate

The public contract has rustdoc examples, the reusable contract suite passes for
the mock backend, dependency boundaries remain clean, all malicious paths are
rejected, and an architecture review freezes the v0 backend API sufficiently for
M10–M13 to proceed.

## Scope boundary

Concrete target syntax, dynamic library loading, network plugin discovery, and
filesystem mutation.
