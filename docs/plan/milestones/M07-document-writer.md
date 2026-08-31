# M07 — Implement structured document writer

- Status: complete
- Phase: 2
- Depends on: M01

## Outcome

Provide deterministic pretty-printing primitives shared by backends without
embedding knowledge of any target language.

## Implementation checklist

- Immutable/composable document nodes for text, line breaks, indent, group, join,
  and conditional layout.
- Width-aware renderer with normalized line endings and final-newline policy.
- APIs that make accidental raw control characters explicit.
- Benchmarks for large/deep documents and configurable depth/size limits.

## Required exit evidence

- Identical documents render identically across hosts.
- The core writer contains no Rust/TypeScript/Python/Go keyword or syntax tables.
- Rendering does not use recursive algorithms that overflow on supported maximum
  depth.
- Backends retain ownership of escaping and identifier rules.

### Verification

- Golden tests for flat/broken groups, nested indentation, empty joins, long
  tokens, Unicode, and final newline.
- Width boundary/property tests.
- Determinism test across repeated renders.
- Limit and deep-document non-overflow tests.
- Benchmark records time and peak allocation for a representative large document.

```text
cargo test -p polyrust-codegen document
cargo bench -p polyrust-codegen document
```

### Completion gate

Golden/property tests pass, benchmark baseline is recorded, public APIs have
examples, and a toy renderer can produce both indentation-sensitive and
brace-based layouts without writer changes.

### Completion evidence

Completed in the pinned Linux development image on 2026-08-31:

- cargo test -p polyrust-codegen document passed eight golden, boundary,
  property-style, determinism, control-character, limit, deep-document, and toy
  layout tests. The public Rustdoc layout example also passed.
- The immutable algebra provides empty/text, soft and hard lines, concat, join,
  indent, group, and conditional break nodes. Normal text rejects controls;
  RawText makes deliberate controls explicit.
- Rendering and fit simulation both use explicit frame vectors. A 4,096-level
  document renders successfully; lowering the configured maximum by one returns
  a structured depth error without recursive traversal.
- Output uses normalized LF endings and exact Preserve, Always, or Never final
  newline policy. Width is deterministic Unicode-scalar count, and long tokens
  are retained intact.
- Repeated renders compare equal in both text and RenderStats. Node, depth, and
  output-byte limits each have an asserted structured failure.
- One writer produces both indentation-sensitive and delimited toy layouts.
  The document module contains no target keyword, escaping, identifier, import,
  or syntax table.
- cargo bench -p polyrust-codegen document rendered 10,000 representative
  declarations. The pinned-image baseline recorded 3,641 microseconds best,
  297,780 output bytes, 425,984 bytes peak output capacity, 20,003 peak pending
  frames, and 140,000 visited nodes.
- docs/document-writer-v0.md specifies the layout and safety contracts and
  records the non-enforcing benchmark baseline.
- Workspace Rustfmt and Clippy passed. bazel test //... passed all 15 repository
  tests across 35 analyzed targets, including the new hermetic document test,
  Rustfmt, Clippy, Buildifier, dependency boundaries, and native generated
  Rust/Go fixture tests.

## Scope boundary

Language escaping, import management, filesystem output, and external formatter
execution.
