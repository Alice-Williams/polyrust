# M07 — Implement structured document writer

- Status: planned
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

## Scope boundary

Language escaping, import management, filesystem output, and external formatter
execution.
