# M06 — Implement typed Rust builder API

- Status: planned
- Phase: 1
- Depends on: M02, M03, M04

## Outcome

Let generator authors construct complete PolyIR modules ergonomically in ordinary
Rust while preserving useful logical source locations and returning diagnostics.

## Implementation checklist

- Builders for modules, constants, records, enums, aliases, contracts,
  implementations, methods, functions, portable tests, types, expressions,
  statements, and patterns.
- Typed IDs/handles where they prevent accidental name/type confusion.
- Logical source paths for builder-created nodes.
- `finish_unchecked` for serialization/testing and `finish` that invokes checking.
- Prelude and rustdoc examples, including the PRD demonstration.

## Required exit evidence

- The demonstration requires no direct construction of internal enum variants.
- Common builder mistakes return diagnostics and do not panic.
- Output from a builder can serialize and be read back into an equal unchecked IR.
- The API does not expose target-specific naming or lowering choices.
- Normal use is possible on stable Rust without procedural macros.

### Verification

- Compile-pass examples for every declaration and expression family, including
  contract implementation and a typed portable test.
- Compile-fail UI tests for typed-handle misuse where the Rust type system should
  reject it.
- Runtime negative tests for duplicate names, missing bodies, and incomplete
  builders.
- Builder → canonical JSON → parser equality.
- Full demonstration checks and evaluates successfully.

```text
cargo test -p polyrust-build
cargo test --doc -p polyrust-build
```

### Completion gate

Rustdoc shows a complete module, compile-fail tests have reviewed messages, no
public function panics on user mistakes, and the example builds the same canonical
IR as a hand-authored fixture and runs its portable tests through the evaluator.

## Scope boundary

Alternative authoring syntax, procedural macros, and code emission.
