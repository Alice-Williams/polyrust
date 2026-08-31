# M06 — Implement typed Rust builder API

- Status: complete
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

### Completion evidence

Completed in the pinned Linux development image on 2026-08-31:

- cargo test -p polyrust-build passed four runtime suites covering the complete
  checked/evaluated demonstration, logical sources, runtime negative cases, and
  compile-pass construction of every declaration, expression, statement,
  pattern, type, value, contract implementation, and typed-test family.
- cargo test --doc -p polyrust-build passed the complete stable-Rust module
  example and two compile-fail typed-handle category tests.
- The demonstration serializes to canonical JSON, parses to an equal Document,
  equals the reviewed hand-authored
  crates/build/testdata/registration.poly.json fixture, checks successfully, and
  passes through the M05 evaluator.
- Constant, alias, record/field, enum/variant/field, contract/method,
  implementation/method, function, and test IDs are distinct Rust handle
  families. Sealed nominal conversion permits only alias, record, and enum
  handles; contract storage is a separate explicit type constructor.
- Missing bodies, missing return types, incomplete contract methods, and
  duplicate declaration names return structured diagnostics. No builder path
  uses a user-triggerable panic.
- Every emitted node has a nonzero deterministic ID and a module-qualified
  logical source path. The API contains no target identifier, syntax, import, or
  lowering concept.
- docs/builder-v0.md documents typed handles, all builder families, and the
  finish_unchecked versus checked finish boundary.
- Workspace Rustfmt and Clippy passed. bazel test //... passed all 14 repository
  tests across 34 analyzed targets, including the new hermetic builder test,
  Rustfmt, Clippy, Buildifier, dependency boundaries, and native generated
  Rust/Go fixture tests.
- The pre-v0 generated registration fixture now imports its explicitly legacy
  IR and checker APIs directly. It remains green until the M10 backend replaces
  that compatibility path and does not leak prototype semantics into M06.

## Scope boundary

Alternative authoring syntax, procedural macros, and code emission.
