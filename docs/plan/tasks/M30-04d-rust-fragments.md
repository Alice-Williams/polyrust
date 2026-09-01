# M30-04D — Migrate Rust to dependency-complete fragments

- Status: in-progress
- Depends on: M30-04C

## Current audit finding

Rust declarations are accumulated into a file-sized source string with imports
attached at file scope. The complete Rust runtime is emitted as a raw text-role
file, bypassing source IR and helper closure.

## Definition of done

- Rust type, value, declaration, test, and support mappings return fragments
  with validated use-path data and helper roots.
- Runtime output is a closed `LanguageSourceFile` assembled from selected helper
  nodes; no generated Rust source uses `LanguageFile::text`.
- Every `use` directive originates exclusively in `RustRenderer`.

## Required tests

- Empty and isolated named type/list/option/result/contract/test/runtime feature
  matrices with exact `use` and helper presence/absence.
- Rustfmt, Clippy, generated crate compile/tests, negative compile tests,
  conformance, and public consumer.
- Three-generation byte determinism.
