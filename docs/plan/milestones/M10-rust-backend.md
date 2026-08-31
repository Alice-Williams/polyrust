# M10 — Implement required Rust backend

- Status: complete
- Phase: 3
- Depends on: M08, M09

## Outcome

Generate a readable, deterministic, safe Rust library crate for the full v0 IR.
This is the first reference source backend and a release-blocking output target.

## Implementation checklist

- Rust name allocator, keyword handling, literal escaping, type/symbol mapping,
  import management, and declaration/expression lowering.
- Cargo package layout and generated tests/runner support.
- Constant lowering, traits, explicit implementations, `&dyn Trait` restricted
  contract parameters, and native `#[test]` generation.
- Semantic helpers only where native Rust behavior is not the PolyRust contract.
- `#![forbid(unsafe_code)]` in generated crates.
- Target mapping and generated API design document.

## Required exit evidence

- Every v0 capability is `native` or a documented `helper`; none is silently
  omitted.
- Generated records/enums, `Option`, `Result`, `i32`/`i64`, strings, bytes, and
  immutable list operations preserve the technical spec.
- Contract conformance is enforced by Rust and every portable test is emitted as
  a discoverable native test.
- Integer behavior never depends on debug/release overflow mode.
- Output has deterministic module/import ordering and passes native formatting.
- The backend uses only the public backend/checker interfaces.

### Verification

- Unit tests for identifiers, raw identifiers, keywords, strings/bytes, special
  floats, types, imports, and every IR lowering case.
- Golden snapshots for complete single- and multi-module crates, including
  constants, trait/impl dispatch, and portable tests.
- Generated compile-fail fixture where a deliberately invalid backend artifact is
  caught by the native compiler (test harness verification).
- Native generated-package checks:

```text
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

- Run generated tests in both debug and release.
- Scan generated source and prove it contains no unsafe blocks/attributes other
  than the required forbid attribute.

### Completion gate

All v0 fixtures generate byte-identically, native checks pass in debug and
release, public API snapshots are reviewed, the backend passes the shared contract
suite, and at least 20 evaluator vectors agree with generated Rust.

## Completion evidence

- `org.polyrust.rust` implements the public checked-program backend contract,
  declares all ten capabilities as native or named helpers, passes the reusable
  backend contract suite, and is registered only in the CLI composition layer.
- Lowering covers every v0 declaration, type, constant, expression, statement,
  pattern, intrinsic, concrete/contract call, and portable-test family. The
  reviewed API mapping is frozen in `docs/rust-backend-v0.md`.
- The generated multi-module crate contains target-owned runtime helpers,
  structs/enums/aliases, traits and explicit impls, `&dyn Trait` parameters,
  exact float bits, escaped strings/bytes, immutable list behavior, and a
  uniform non-panicking `PolyResult<T>` evaluation-outcome ABI.
- Generation is byte-identical across repeated runs. The reference evaluator
  passes every declared checked-fixture test before generation; the generated
  crate emits that portable test plus all 20 shared semantic vectors.
- Bazel generates and compiles the Rust source through the pinned toolchain. Its
  native gate runs the explicit `cargo fmt` post-process and check, Clippy with
  warnings denied, and debug/release tests (21 pass in each mode), scans the
  whole source tree so the only unsafe token is `#![forbid(unsafe_code)]`, and
  proves a deliberately unsafe generated artifact fails compilation.
- A real `polyrust emit --target org.polyrust.rust` safe-output run generated the
  four-file crate and passed native Clippy plus all release tests.
- The complete repository gate passes Cargo formatting/Clippy/tests/doctests and
  all 19 Bazel tests, including Rustfmt, Clippy, Buildifier, dependency
  boundaries, generated compilation, native checks, and compile-fail proof.

## Scope boundary

Generating binaries, arbitrary Cargo dependencies, async, unsafe code, FFI, and
parsing Rust input.
