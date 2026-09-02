# M34A-13 — Migrate TypeScript to typed generation

- Status: planned
- Depends on: M34A-12

## Goal

Make strict TypeScript the single typed executable source for both TypeScript
and derived JavaScript packages.

## Definition of done

- TypeScript owns all type, expression, statement, binding/pattern,
  declaration, class/interface member, import/export, file/package, and
  template enums in its specification.
- Runtime-erasure classification is explicit on every relevant node.
- Every CoreIR feature has an exhaustive TypeScript strategy; ECMAScript
  globals and Node module callables have exact typed catalogue entries.
- Fixed-width number/bigint operations, exact F64 bits, Unicode, immutable
  readonly/copy-safe values, tagged unions, failures, and evaluation order are
  verifier checked.
- Flat interfaces, multiple conformance, first-class/nested interface values,
  runtime witnesses where needed, dynamic dispatch, and readonly composition
  pass without `extends`, mixins, declaration merging, or prototype mutation.
- Imports/exports, type-only edges, module paths, helpers, declaration shims,
  and file roles are resolver-derived.
- Runtime declarations are TypeScript AST; strict templates render resolved
  TypeScript views.
- Paired/raw `EcmaCode`, checked-in executable runtime strings, manual import/
  helper metadata, and independent JavaScript branches are deleted.
- The TypeScript compliance row moves to **Pass** with exact evidence.

## Tests

- `bazel test //crates/backend-typescript:typescript_all --nocache_test_results --test_output=errors`
- TypeScript AST/verifier/catalogue/import/helper/module/template/erasure
  positive and negative matrices.
- Pinned Prettier no-diff, strict `tsc --noEmit`, native Node tests, negative
  type fixtures, interface corpus, and three-generation determinism.
- All M17-M33 TypeScript historical port targets.

## Commit gate

Commit and push `M34A-13: migrate TypeScript to typed AST` only when the
TypeScript package is green and contains no independent JavaScript source path.
