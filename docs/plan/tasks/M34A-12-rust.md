# M34A-12 — Migrate Rust to typed generation

- Status: planned
- Depends on: M34A-11

## Goal

Generate Rust through a complete typed Rust AST with resolver-derived paths and
structural helpers, while preserving strict Rust lint and safety guarantees.

## Definition of done

- Rust owns all type/path, expression, statement, pattern, item, attribute,
  visibility, generic, file/package, and template enums in its specification.
- Every CoreIR feature has an exhaustive Rust strategy; all known `std`
  types/methods/macros are typed by namespace and exact signature.
- Ownership/cloning, evaluation order, match exhaustiveness, paths,
  visibility, object safety, and precedence are verified before rendering.
- Flat traits/explicit impls, multiple conformance, owned immutable
  `Arc<dyn Trait>` interface wrappers, nested interface values, static/dynamic
  dispatch, and explicit delegation pass with no supertraits or promotion.
- Resolver-derived `use` items, modules, helper closure, visibility, and files
  are exact under collision fixtures.
- Runtime items are Rust AST and strict Rust Handlebars renders resolved views.
- `RustCode`, executable `LanguageFragment`/document construction, raw runtime
  source, manual use/helper metadata, and the legacy Rust pipeline are deleted.
- Generated crates contain `#![forbid(unsafe_code)]` and no unsafe escape.
- The Rust compliance row moves to **Pass** with exact evidence.

## Tests

- `bazel test //crates/backend-rust:all --nocache_test_results --test_output=errors`
- Rust AST/verifier/catalogue/use/helper/module/template and compile-fail
  matrices.
- Generated Rustfmt check, Clippy all-targets with warnings denied, debug and
  release tests, external consumer, unsafe source scan, interface corpus, and
  three-generation determinism.
- All M17-M33 Rust historical port targets.

## Commit gate

Commit and push `M34A-12: migrate Rust to typed AST` only when all Rust and
shared typed-generation gates pass in the dev container.
