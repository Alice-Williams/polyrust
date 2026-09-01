# M30-04B — Migrate Python to dependency-complete fragments

- Status: planned
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
