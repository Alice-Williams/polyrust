# M30-04F — Migrate C to dependency-complete fragments

- Status: planned
- Depends on: M30-04E

## Current audit finding

C generation returns complete header/source/test strings and attaches
dependencies only at file scope. Runtime header/source files use fixed include
inventories, so individual ABI/support mappings cannot prove closure.

## Definition of done

- C ABI type, declaration, definition, ownership helper, portable-test, and
  runtime mappings return fragments owning validated system/local includes and
  helper roots.
- Header guards remain structured preamble/epilogue fragments; only `CRenderer`
  spells `#include`.
- Runtime ownership and semantic helpers form a deterministic graph with no
  unrelated nodes in a minimal program.

## Required tests

- Empty and isolated ABI/string/bytes/list/option/result/numeric/runtime matrices
  with exact includes and helper closure.
- Header self-containment, C17 warnings-as-errors, ABI shape checks, ownership
  tests, ASan/UBSan, conformance, and public consumer.
- Three-generation byte determinism.
