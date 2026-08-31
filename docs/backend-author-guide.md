# Backend author guide

A backend is an extension over public, versioned crates: `polyrust-codegen`
provides `Backend`, registry, preflight, manifest, and contract-test APIs;
`polyrust-check` provides `v0::CheckedProgram`; and `polyrust-ir` provides the
versioned `v0` nodes and capabilities. Backends must not depend on emitter
internals or add a target-name branch to the core.

Use the separately rooted
[`examples/external-backend`](../examples/external-backend/Cargo.toml) package as
the template. Its [implementation](../examples/external-backend/src/lib.rs):

1. defines a namespaced `TargetId` and supported `IrVersionRange`;
2. reports support for every capability before generation;
3. declares its option schema;
4. consumes only `CheckedProgram` and returns an in-memory `OutputManifest`;
5. registers through `BackendRegistry`, explicitly preflights, generates, and
   calls `check_backend_contract` to prove deterministic behavior.

Run the template proof with:

```sh
bazelisk test //examples/external-backend:external_backend_test
```

Before proposing a real backend, add target-native compilation and tests for
every generated portable test, negative tests for unsupported capabilities and
invalid options, deterministic manifest tests, and the shared backend contract
test. Follow the existing [Rust](rust-backend-v0.md),
[TypeScript](typescript-backend-v0.md), [Python](python-backend-v0.md), and
[Go](go-backend-v0.md) backend documents for target-specific precedents.

