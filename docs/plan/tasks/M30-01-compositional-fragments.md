# M30-01 — Add compositional language fragments

- Status: complete

## Goal

Make target syntax and its import requirements inseparable at declaration,
expression, type, and helper granularity.

## Definition of done

- `LanguageFragment<Import>` owns one `Document`, one `ImportSet<Import>`, and
  its runtime-helper roots.
- Sequence, optional, mapped, joined, and nested composition merge all parts.
- A fragment converts to a `LanguageUnit` without exposing a second import path.
- Existing package/file rendering remains syntax-only and target-independent.
- Public documentation distinguishes fragments, units, files, and renderers.

## Tests

- Empty, single, nested, optional, and joined fragment composition.
- Associativity and deterministic import ordering/deduplication.
- Compile-fail coverage for appending a naked document to a translated unit.
- `bazel test //crates/codegen:all //:rustfmt_test //:rust_clippy_test`
