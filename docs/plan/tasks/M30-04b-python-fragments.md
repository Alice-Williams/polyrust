# M30-04B — Migrate Python to dependency-complete fragments

- Status: complete
- Depends on: M30-04A

## Current audit finding

Python declaration text and imports mutate one file-sized unit. A separate type
walk reconstructs `typing`, `dataclasses`, and runtime imports after syntax
selection. The complete runtime uses a fixed inventory.

## Definition of done

- Python type, value, declaration, decorator, test, and runtime mappings return
  fragments that own future/module/from imports.
- The `require_type` and declaration-wide repair paths are deleted.
- Runtime support is a helper graph with exact standard-library imports.
- `PythonImport` validates modules/names and the renderer alone spells `import`
  and `from` directives.

## Required tests

- Empty and isolated alias/dataclass/enum/protocol/callable/option/result/F64
  matrices with exact positive and negative imports.
- Runtime helper matrices for every imported standard-library module.
- Ruff or configured Python lint, type checking, native tests, conformance, and
  public consumer.
- Three-generation byte determinism.

## Completion evidence

- `PythonCode` fragments compose validated future, module, and from-import
  requirements through nested types, values, declarations, and portable tests.
- `PythonImport` stores only validated semantic data; the renderer alone spells
  Python import directives. Direct imports reject relative modules while
  from-imports permit validated relative paths.
- The declaration-wide `require_type` and import-repair paths are deleted.
  Nested `Result<Option<I64>, String>` coverage proves local dependency
  propagation.
- The Python runtime is an ordered helper graph. Common helpers and optional
  F64 helpers own exact `dataclasses`, `types`, `typing`, `math`, and
  `struct` imports; helper metadata is absent from rendered output.
- Backend unit tests, generated-package Ruff/type/native/conformance/public
  checks, Rustfmt, and Clippy pass. All 130 tests in
  `//examples/real-world/...` pass, including the F64-heavy `parse-ms`
  translation.
