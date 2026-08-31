# M02 — Define versioned portable IR and canonical serialization

- Status: complete
- Phase: 1
- Depends on: M01

## Outcome

Implement the complete unchecked v0 IR, source references, version header, and
canonical `.poly.json` representation without target-language concepts.

## Implementation checklist

- Rust types for modules, constants, aliases, records, enums, contracts,
  implementations/methods, functions, portable tests, expressions, statements,
  node IDs, and logical/file source references.
- `IrVersion` parsing and compatibility checks.
- Canonical JSON reader/writer with limits for bytes, depth, nodes, and strings.
- Fixtures for every v0 node and deliberately unsupported/unknown versions.
- `docs/ir-v0.md` normative schema and compatibility rules.

## Required exit evidence

- Every Core construct in the portable language map is representable, including
  restricted contract parameters and typed expected test outcomes.
- There are no `RustType`, `GoType`, or other target-specific variants.
- Equivalent IR serializes identically regardless of map/hash insertion order.
- Unknown fields and unsupported major versions return structured placeholder
  errors ready to be converted to M03 diagnostics; no panic occurs.
- Duplicate `NodeId` values are detectable by a structural validation pass.

### Verification

- Unit tests for every type/node constructor and version comparison.
- Golden canonical JSON for a module using every node category, including a
  contract, implementation, and portable test.
- Parse → serialize → parse equality property tests.
- Randomized declaration insertion order produces identical canonical bytes.
- Limits reject excessive depth, node count, string length, and total bytes.
- Fuzz/property inputs never panic in the JSON reader.

```text
cargo test -p polyrust-ir
cargo test -p polyrust-ir --all-features
```

### Completion gate

The exhaustive fixture round-trips, determinism/property tests pass, the schema
is documented, public types have rustdoc examples, and target backends can depend
on the model without pulling parser/CLI dependencies.

### Completion evidence

Completed in the pinned Linux development image on 2026-08-31:

- `cargo test -p polyrust-ir` and
  `cargo test -p polyrust-ir --all-features` passed 10 unit/property tests and
  the public v0 rustdoc example.
- `bazel test //...` passed all 10 repository tests across 30 analyzed targets,
  including the hermetic IR test, Rustfmt, Clippy, Buildifier, dependency
  boundaries, and native generated Rust/Go tests.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` and
  `cargo fmt --all --check` passed.
- The 6.7 KiB `every-node.poly.json` golden fixture covers every declaration
  category, all Core types, a contract, implementation, restricted contract
  parameter, and typed portable test outcome.
- Dedicated tests round-trip every expression, statement, pattern, constant,
  value, intrinsic, invocation, outcome, and source-reference variant.
- Deterministic shuffles prove declaration and metadata insertion order cannot
  change canonical bytes; parse/serialize/parse equality is asserted.
- Strict-schema tests reject root and nested unknown fields and incompatible
  majors with structured error kinds. Structural tests reject zero/duplicate
  `NodeId` values.
- Byte, depth, structural-node, and string limits each have rejection tests;
  512 deterministic malformed byte inputs are caught as errors without panic.
- `docs/ir-v0.md` is the normative schema. The IR Cargo package depends only on
  locked Serde/Serde JSON plus the standard library and does not depend on a
  parser, CLI, checker, or backend.

## Scope boundary

Name resolution, type correctness, evaluation, custom textual DSL, and source
language parsing.
