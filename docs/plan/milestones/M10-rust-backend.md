# M10 — Implement required Rust backend

- Status: planned
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

## Scope boundary

Generating binaries, arbitrary Cargo dependencies, async, unsafe code, FFI, and
parsing Rust input.
