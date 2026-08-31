# M03 — Implement structured diagnostics and source references

- Status: complete
- Phase: 1
- Depends on: M01

## Outcome

Provide a stable, testable diagnostic model used by parsing, checking, backends,
and output safety checks.

## Implementation checklist

- `Diagnostic`, `DiagnosticCode`, severity, labels, related locations, notes, and
  remediation hints.
- File-span and logical builder-path source references.
- Human terminal renderer with optional color and a machine-readable JSON
  renderer.
- Initial code registry covering the codes listed in the technical spec.
- `explain(code)` data/API and collision test for registered codes.

## Required exit evidence

- User-controlled invalid input cannot trigger a panic.
- Diagnostics sort deterministically by source then code.
- Rendering handles Unicode filenames/text and missing source content.
- JSON rendering is stable and contains no ANSI escapes.
- Code creation is centralized; duplicate codes fail a test.

### Verification

- Golden tests for plain, colored, multi-label, logical-path, and missing-source
  diagnostics.
- JSON schema/snapshot tests.
- Unicode and zero-width span cases.
- Property test that arbitrary safe spans cannot cause slicing panics.
- Registry uniqueness and `explain` coverage tests.

```text
cargo test -p polyrust-diagnostics
```

### Completion gate

All renderers pass snapshots on Windows and Unix newline modes, every registered
code has a short and long explanation, and another crate can construct and test a
diagnostic without depending on terminal UI code.

### Completion evidence

Completed in the pinned Linux development image on 2026-08-31:

- `cargo test -p polyrust-diagnostics` passed 9 unit, golden, schema,
  determinism, and generated-input tests plus doc tests.
- `bazel test //...` passed all 12 repository tests across 32 analyzed targets,
  including the hermetic diagnostics and downstream-consumer tests, Rustfmt,
  Clippy, Buildifier, dependency boundaries, and native generated Rust/Go tests.
- Plain and ANSI terminal goldens cover Unicode text, notes, hints, and target
  context. Additional goldens cover multiple labels, related locations, logical
  builder paths, and unavailable source content.
- The stable JSON snapshot asserts every top-level field and contains no raw ANSI
  escape bytes. `docs/diagnostics.md` records the JSON shape and initial code
  registry.
- LF and CRLF render modes are compared for identical normalized content.
- Unicode, zero-width, reversed, mid-scalar, oversized, and `u64::MAX` spans
  are clamped before slicing. A deterministic generated-input test renders 2,048
  arbitrary span pairs without panic.
- Diagnostics are tested to sort by source and then code. The centralized
  registry test rejects duplicate strings and requires distinct non-empty short
  and long explanations for every registered code.
- `//smoke/diagnostics:diagnostics_consumer_test` imports only the public code,
  model, and source-reference API to construct and test a diagnostic from a
  separate crate; the diagnostics package has no terminal UI dependency.

## Scope boundary

Checker rules and language-specific reserved-name diagnostics.
