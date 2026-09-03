# Rust typed-generation specification

- Status: normative for M34A
- Target ID: `org.polyrust.rust`
- Language/toolchain: Rust 2024 edition, Rust 1.98.0

## Inferred typed-program admission

`RustDialect` MUST implement `Supports<F>` separately for each portable
feature whose complete Rust mapping has passed this specification. Typed
generation accepts `TypedProgram<R>` only under
`RustDialect: SupportsAll<R>`. There is no profile-wide, wildcard, or default
support claim; before this migration is complete, absent implementations make
the typed call fail during generator compilation.

## 1. Scope and package

The plugin emits a dependency-free library crate with:

- `Cargo.toml`;
- `src/lib.rs`;
- `src/polyrust_runtime.rs` when helpers are selected;
- `src/conformance.rs`; and
- native tests derived from portable tests.

Public/package visibility maps to `pub`/`pub(crate)`. Generated code contains
`#![forbid(unsafe_code)]` and no other `unsafe` token.

## 2. Capability strategies

The exhaustive Rust capability registry uses strategy enums. Native strategies
cover fixed-width primitives, options/results, records/enums, interfaces,
pattern matching, bounded iteration, and most operations. Emulated strategies
select structural runtime helpers for semantics not matched exactly by a Rust
operator or standard method.

Every CoreIR feature is explicitly `Native`, `Emulated`, or `Unsupported`.
There is no default arm.

## 3. Rust AST

The dialect owns distinct enums/types for:

- `RustType` and `RustTypePath`;
- `RustExpr`;
- `RustStmt`;
- `RustPattern`;
- `RustItem`;
- `RustAttribute` and `RustVisibility`;
- `RustGenericArgument`;
- `RustFile`; and
- closed Rust grammar and formatting categories.

Expressions retain a `RustPrecedence` enum. Operators, attributes, visibility,
reference kinds, mutability, and item kinds are enums rather than strings.

Known primitive mapping builders use phantom-typed `Expr<RustDialect, T>`
handles. Generated nominal declarations use typed IDs plus verifier checks.

## 4. Type mapping

| CoreIR type | Rust representation |
| --- | --- |
| Unit | `()` |
| Bool | `bool` |
| I32 / I64 | `i32` / `i64` |
| F64 | `f64` reconstructed from exact bits |
| Char | `char` |
| String | owned `String` |
| Bytes | owned `Vec<u8>` |
| List<T> | owned `Vec<T>` with immutable public behavior |
| Option<T> | `Option<T>` |
| Result<T,E> | `Result<T,E>` |
| Record | generated nominal `struct` |
| Tagged enum | generated `enum` with named payload fields |
| Interface | generated owned interface-value newtype around `Arc<dyn Trait>` |

`Arc` sharing is not observable because portable interface values expose no
identity or mutation. Underlying record values are immutable through the trait.

## 5. Declarations and control

- Constants use native `const` only when legal; otherwise a private stable
  value/accessor strategy is selected explicitly.
- Records have private or visibility-controlled fields and generated
  constructors/accessors needed by the portable API.
- Tagged enums use exhaustive Rust matching.
- Functions and methods return the generated callable-result shape required by
  runtime failure semantics.
- CoreIR left-to-right order is preserved with explicit temporaries because
  Rust argument evaluation order is not used as an undocumented assumption.
- Ownership-consuming calls insert CoreIR-required clones only.

## 6. Interfaces and composition

A portable interface lowers to a flat Rust trait with immutable `&self`
methods. A record implementation lowers to `impl Trait for Record`. Multiple
independent trait implementations are allowed; supertraits are not generated.

An interface value is a generated nominal wrapper around `Arc<dyn Trait>`.
Coercion constructs an immutable `Arc`. Calls borrow the wrapper once and
dispatch through the trait object.

Composition uses ordinary struct fields and explicit delegation. The plugin
does not use inheritance, trait superchains, `Deref`-based member promotion, or
blanket implementations to create portable behavior.

## 7. Symbols and imports

`RustKnownType`, `RustKnownCallable`, `RustKnownMacro`, and
`RustRuntimeCallable` are closed enums. Catalogue entries own full module
paths, namespaces, signatures, visibility, and import policy.

The initial catalogue includes all selected `std` types/methods for owned
strings, vectors, `Arc`, binary64 bit conversion, UTF-8 handling, and tests.

The resolver:

- distinguishes type/value/macro/module namespaces;
- derives `use` items from known symbol references;
- leaves prelude symbols unimported;
- qualifies colliding external symbols;
- preserves public portable names;
- allocates deterministic private suffixes; and
- creates no external Cargo dependency for `std`.

No lowering code creates a `use` directive or import record.

## 8. Runtime helpers

`RustRuntimeHelper` is a closed enum. Each helper is Rust AST declarations and
typed references. The minimum runtime contains only permanently required
callable-result/error definitions; optional helpers are selected per use.

No helper is stored in a Rust source string, `Document`, token stream, or
feature-specific source template.

## 9. File and package policy

Typed file roles select the crate manifest, crate root, runtime module,
conformance module, and native tests. The resolver owns module declarations,
item placement, visibility paths, declaration order, and deterministic file
names. Module cycles or public signatures which expose private generated
symbols are rejected before rendering.

## 10. Rendering

The Rust post-link checker certifies complete modules as an opaque
`RenderReadyPackage<RustDialect>`. It validates item/module placement,
namespaces, visibility, attributes, generic and trait/impl shape, pattern and
binding scopes, ownership-sensitive expression forms admitted by the dialect,
control-flow context, and return/termination rules for the pinned edition.

The total Rust renderer structurally covers files, attributes, use
declarations, structs, enums, traits, impls, functions, methods, generics,
blocks, statements, patterns, expressions, literals, and documentation. It owns
Rust precedence, keyword/raw-identifier policy, literal escaping, doc-comment
spelling, and all fixed tokens through exhaustive AST matches. Special path
keywords which cannot be raw identifiers receive the documented deterministic
escape. There is no executable Handlebars template or token/source escape
hatch.

Rustfmt, Clippy, and rustc are independent pinned or hermetic oracles. The
unformatted source MUST already parse and compile; Rustfmt cannot create the
certificate or make invalid syntax valid.

## 11. Validation

The Rust AST verifier checks:

- type/call/method signatures;
- trait method ownership and object safety;
- no supertraits or target heritage;
- borrow/ownership and clone marker consistency;
- match exhaustiveness as represented;
- visibility and module paths;
- macro namespace references;
- precedence classifications; and
- no opaque executable nodes.

## 12. Success evidence

- Exhaustive AST constructor/unit/compile-fail tests.
- Catalogue signature/import and collision matrices.
- Interface declaration, multiple implementation, owned interface value,
  nested interface container, static/dynamic dispatch, and delegation tests.
- Exact helper presence/absence matrices.
- Three-generation AST/render/manifest determinism.
- Generated `cargo fmt --check`.
- Generated `cargo clippy --all-targets -- -D warnings`.
- Generated debug and release `cargo test`.
- Negative compile fixtures for type and unsafe violations.
- Complete source scan proving `unsafe` is forbidden.
- Every historical port and the canonical conformance corpus.

## 13. Migration exit

The Rust language passes only after `RustCode`/`LanguageFragment` executable
text construction and raw runtime source are deleted, all accepted features
use `RustAst`, and the repository/hosted gates are green.
