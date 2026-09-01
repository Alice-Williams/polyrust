# M30-04E — Migrate C++ to dependency-complete fragments

- Status: in-progress
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
