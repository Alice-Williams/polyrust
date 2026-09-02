# M34A-15 — Migrate Python to typed generation

- Status: planned
- Depends on: M34A-14

## Goal

Generate strictly typed, immutable Python packages through Python AST and
resolver-derived imports.

## Definition of done

- Python owns all type, expression, precedence, statement, pattern,
  declaration, decorator, visibility, file/package, and template enums in its
  specification.
- Every CoreIR feature has an exhaustive Python strategy; builtins and exact
  admitted `dataclasses`, `typing`, `math`, `struct`, and `types` callables
  are catalogued.
- Fixed-width integer, exact F64 bits, Unicode, tagged values, frozen records/
  collections, failures, and evaluation order are verified without unbounded
  `Any`.
- Flat Protocols, exact implementations, multiple conformance, first-class/
  nested interface values, dynamic dispatch, and dataclass-field delegation
  pass with no mixins, inherited implementation, monkey patching, or promotion.
- Module/from/relative/type-only imports, aliases, exports, helpers, files, and
  package initializers are resolver-derived.
- Runtime declarations are Python AST; strict templates render resolved views.
- Raw Python/body/runtime constants, manual imports/helpers, and the legacy
  Python pipeline are deleted.
- The Python compliance row moves to **Pass** with exact evidence.

## Tests

- `bazel test //crates/backend-python:all --nocache_test_results --test_output=errors`
- Python AST/verifier/catalogue/import/helper/module/template matrices.
- Python compileall, Ruff format/check, strict mypy with negative fixtures,
  pytest native/conformance tests, interface corpus, immutable aliasing/
  surrogate boundaries, and three-generation determinism.
- All M17-M33 Python historical port targets.

## Commit gate

Commit and push `M34A-15: migrate Python to typed AST` only when all Python
and shared typed-generation gates pass in the dev container.
