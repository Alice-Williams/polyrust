# Rust backend v0 mapping

Status: required backend contract for M10.

## Package and API shape

The backend target is `org.polyrust.rust`. It emits a library crate containing
`Cargo.toml`, `src/lib.rs`, `src/polyrust_runtime.rs`, and
`src/conformance.rs`. The generator is deterministic and pure; `cargo fmt` is an
explicit pinned post-process before native lint/test execution.

Public/package visibility maps to `pub`/`pub(crate)`. Records are immutable
owned structs, tagged enums are Rust enums with named payload fields, aliases
are type aliases, restricted contracts are traits, and explicit implementations
are trait impls. Contract parameters use `&dyn Trait`; other parameters use
owned value semantics. String, bytes, and list values are owned `String`,
`Vec<u8>`, and `Vec<T>` values and are cloned only where checked expression use
requires non-consuming immutable semantics.

Rust keywords use raw identifiers where Rust permits them. The special path
keywords `self`, `Self`, `super`, and `crate` receive a trailing underscore.
Names are otherwise preserved, with target lint allowances for portable naming
styles. Documentation is emitted only through prefixed Rust doc comments.

## Types

| PolyRust | Generated Rust |
| --- | --- |
| `Unit`, `Bool` | `()`, `bool` |
| `I32`, `I64`, `F64` | `i32`, `i64`, `f64` |
| `Char`, `String`, `Bytes` | `char`, `String`, `Vec<u8>` |
| `List<T>` | `Vec<T>` with non-aliasing helper lowering |
| `Option<T>` | `Option<T>` |
| `Result<T,E>` | `Result<T,E>` |
| named record/enum | generated nominal struct/enum |
| restricted contract parameter | `&dyn GeneratedTrait` |

Every generated function and trait method returns `PolyResult<T>`. This uniform
ABI reflects the implemented v0 evaluator model: expression types describe the
success value while checked arithmetic, indexing, narrowing, and UTF-8 decoding
can produce an evaluation error. Native lowering propagates those errors with
`?`; it does not use exceptions or panics as portable control flow.

## Capabilities and helpers

Bytes, floating point, contract dispatch, option/result, wrapping arithmetic,
and bounded iteration use native Rust semantics. Checked integer operations,
Unicode scalar operations, and immutable-list operations are declared helper
capabilities. Their code lives in `polyrust_runtime.rs` and returns the stable
`PolyRuntimeError` categories used by the evaluator.

`i32`/`i64` checked and wrapping methods make behavior identical in debug and
release. Scalar text operations use `.chars()`. Float literals use exact
`f64::from_bits`, preserving NaN payloads and negative zero. Strings and bytes
are escaped/encoded by the backend; no target source from the input is accepted.

## Test and safety contract

Every portable test becomes a discoverable `#[test]`. Each crate also contains
20 native semantic vectors matching the evaluator corpus. The Bazel native gate
runs formatting, Clippy with warnings denied, and tests in debug/release, scans
the complete generated source tree so the only `unsafe` token is
`#![forbid(unsafe_code)]`, and verifies a deliberately unsafe backend artifact is
rejected by the compiler.
