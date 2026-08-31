# M03 — Implement structured diagnostics and source references

- Status: planned
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

## Scope boundary

Checker rules and language-specific reserved-name diagnostics.
