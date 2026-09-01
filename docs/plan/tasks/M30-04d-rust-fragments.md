# M30-04D — Migrate Rust to dependency-complete fragments

- Status: complete
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

## Completion evidence

- Validated private `RustImport` data distinguishes module declarations and
  use paths, including test-only and public status. Only `RustRenderer`
  spells `mod` and `use`; invalid names, paths, and directive text are
  rejected.
- `RustCode` composition carries imports and helper roots through nested
  types, values, constants, expressions, blocks, calls, patterns,
  declarations, portable tests, documentation, and conformance support. No
  target-syntax mapping returns a naked string.
- The runtime is a closed source file assembled by deterministic helper
  closure. Replacement, byte replacement, UTF-8 truncation, and checked-shift
  roots select exact nodes; the permanent conformance core remains minimal.
- Exact import/type/helper matrices and direct intrinsic-root assertions pass.
  Helper markers never enter generated output.
- Three-generation determinism, generated-crate Rustfmt and Clippy,
  debug/release tests, negative compilation, backend tests, and all 130 tests
  in `//examples/real-world/...` pass.
