# M26 — Dependency-bearing flat language IR

- Status: complete
- Phase: 8
- Depends on: M24, M25

## Problem statement

M24 introduced explicit packages, file groups, source files, and structured
imports, but left one important escape hatch: a plugin could attach imports to a
file independently from the target syntax that uses them, then install an
already-rendered `Document` as the body. That makes the package envelope useful,
but it does not yet make each portable-to-target mapping self-contained.

The intended pipeline is:

```text
CheckedProgram
    -> target plugin / flat mappings
    -> dependency-bearing target language units
    -> grouped target language package IR
    -> syntax-only renderer
    -> OutputManifest
```

## Outcome

Every translated source section is a flat language unit containing target
syntax and the complete import/include/use requirements caused by that syntax.
Adding a unit to a target file dynamically merges its requirements. A source
file cannot accept an import independently, so a mapping cannot emit syntax and
forget or predeclare its dependencies in a separate static import section.

The renderer remains intentionally small: order and render the already-collected
imports, render the translated documents, join the file sections, and enforce
output limits. It cannot inspect checked PolyIR or select imports.

## Implementation checklist

- Add a public `LanguageUnit<Import>` target-IR node that owns one translated
  syntax document and a sorted, deduplicated import requirement set.
- Make `LanguageSourceFile` accept units for preamble, body, and epilogue and
  remove its independent import mutation API.
- Merge unit requirements automatically when a unit is installed in a file.
- Preserve explicit deterministic file groups and file roles from M24.
- Migrate Rust, TypeScript, JavaScript, Python, Go, Java, C++, and C so every
  import requirement is attached to the translated unit that uses it.
- Keep checked runtime/support assets inside the same contract: their fixed
  dependencies belong to their runtime unit, never to renderer source text.
- Update the architecture and backend author guide to document the strengthened
  plugin boundary and the distinction between language IR and final source.

## Required exit evidence

- Unit tests prove imports from multiple units are merged, sorted, deduplicated,
  and omitted when no translated unit requests them.
- Compile-fail/API tests prove `LanguageSourceFile` has no independent
  `require_import` path and the renderer cannot inspect `CheckedProgram`.
- Focused backend tests prove conditional imports still appear and disappear
  with the mapped constructs that require them for all eight targets.
- Three complete generations remain byte-identical.
- Every generated package passes its formatter/linter/static, native compiler,
  portable tests, and conformance tests in the Linux development container.
- `bazelisk test //... --test_output=errors` and
  `bazelisk test //:release_gate --test_output=errors` pass, including
  Buildifier, Rustfmt, and Clippy.

## Scope boundary

M26 makes flat mapping units dependency-complete and removes independent import
assembly. Their syntax payload continues to use the immutable structured
`Document` algebra; later language plugins may deepen individual declaration or
expression ASTs without changing unit dependency ownership, file grouping, or
manifest rendering.

## Completion evidence

- `polyrust-codegen` now exposes `LanguageUnit<Import>`, which owns one target
  syntax document and its sorted import requirements. `LanguageSourceFile`
  accepts units for its preamble, body, and epilogue and no longer exposes a
  file-level `require_import` method.
- The package renderer derives a file's import set exclusively by merging its
  units. Core tests prove cross-unit merge, ordering, deduplication,
  empty-import omission, and renderer error propagation. A compile-fail example
  locks out independent file import mutation.
- Rust, TypeScript, JavaScript, Python, Go, Java, C++, and C all translate
  generated source, runtime/support source, portable tests, conformance tests,
  and negative tests through dependency-bearing units. Conditional type and
  test imports remain tied to the unit mapping that selected them.
- Focused Bazel backend suites passed for every target. These include generated
  native packages, Java/C/C++ public API consumers, C ABI and ownership tests,
  target style gates, and C/C++ sanitizer tests.
- The complete uncached Linux-container repository gate passed **153/153**
  tests. It included every native and differential real-world proof,
  three-generation determinism, Buildifier, Rustfmt, Clippy, documentation,
  dependency boundaries, and release policy checks.
- The separate uncached Linux-container release gate passed **131/131** tests,
  including all target linters/formatters, native compilers, conformance tests,
  public consumers, sanitizers, differential oracles, and determinism gates.
