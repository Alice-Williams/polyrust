# M30-04E — Migrate C++ to dependency-complete fragments

- Status: complete
- Depends on: M30-04D

## Current audit finding

C++ header/source declarations are file-sized strings. Header requirements are
reconstructed by separately scanning declarations and capabilities. The runtime
unconditionally attaches a seventeen-header inventory.

## Definition of done

- C++ type, value, declaration, definition, test, and runtime mappings return
  fragments owning validated system/local include keys and helper roots.
- Header and capability repair scans plus the fixed runtime inventory are
  deleted.
- Declarations and matching definitions compose without losing requirements;
  only `CppRenderer` spells `#include`.

## Required tests

- Empty and isolated STL type/container/numeric/string/variant/runtime matrices
  with exact includes and forward declarations.
- Header self-containment, formatter/linter, generated compile/tests,
  conformance, and public consumer.
- Three-generation byte determinism.

## Completion evidence

- Validated `CppImport` values distinguish system and local includes; only the
  renderer spells directives.
- `CppCode` fragments own and compose exact includes and helper roots across
  types, declarations, definitions, conversion bridges, tests, and runtime
  bootstrap syntax. The header repair scans and fixed runtime inventory are
  removed.
- Marked runtime model, JSON, and engine nodes declare exact header matrices;
  `runtime.full` is reached from the source fragment's helper root.
- Backend unit/native/conformance/public/style/sanitizer tests, Rustfmt,
  Clippy, Buildifier, release policy, and all 130 real-world tests pass.
- The unit suite proves three independently generated manifests are identical.
