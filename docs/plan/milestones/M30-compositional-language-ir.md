# M30 — Compositional target-language IR

- Status: complete
- Phase: 8
- Depends on: M26, M29

## Problem statement

M26 made imports properties of file-sized `LanguageUnit` values, so renderers no
longer inspect checked PolyIR or prepend import blocks. Some plugins still build
a large body string and maintain its requirements through a parallel scan or a
fixed runtime import inventory. Java currently demonstrates both remaining
risks: `Generated.java` requests some imports after separately scanning its
declarations, and `Runtime.java` has one manually maintained import list for a
monolithic runtime body.

The stricter pipeline is:

```text
CheckedProgram
    -> target plugin
    -> dependency-bearing type/expression/declaration/helper fragments
    -> composed target files and runtime-helper dependency closure
    -> syntax-only renderer
    -> OutputManifest
```

No mapping may return naked target source while requiring its caller to remember
an import. Composition must merge syntax and requirements as one operation.

## Outcome

Every target mapping returns a compositional fragment containing its target
syntax and structured import requirements. File bodies are folds over those
fragments. Runtime/support code is selected from a target-owned helper graph,
whose nodes carry their own syntax, imports, and helper dependencies. Renderers
only format the already-resolved package IR.

Java is the first complete vertical slice. The other seven plugins then migrate
to the same invariant without sharing target syntax or target-specific ASTs.

## Implementation checklist

- Freeze a normative architecture contract and evidence-backed baseline audit
  for shared codegen plus all eight outputs.
- Add a compositional `LanguageFragment<Import>` abstraction with deterministic
  document concatenation and import merging.
- Make fragment composition the normal path into a `LanguageUnit`; retain no
  API that appends a naked `Document` to an existing translated body.
- Give each backend a structured import key rather than an unvalidated source
  line.
- Migrate Java type, expression, declaration, test, and file mappings so every
  mapping returns a dependency-complete fragment.
- Split Java runtime/support code into dependency-bearing helper nodes and emit
  the transitive helper closure selected by the checked program.
- Remove Java's declaration pre-scan and fixed runtime import inventory.
- Migrate Rust, TypeScript/JavaScript, Python, Go, C++, and C using the proven
  Java pattern.
- Add policy tests that reject import/include/use directives in generator body
  templates outside the target renderer and explicitly admitted handwritten
  consumer fixtures.

## Required exit evidence

- Every row in the compliance ledger is `Pass`; compilation alone is not
  accepted as dependency-completeness evidence.
- Core tests prove fragment composition is associative, deterministic, sorted,
  deduplicated, and cannot lose requirements when nested.
- Java matrix tests independently toggle every import-bearing construct and
  helper, proving exact import presence and absence.
- A minimal Java program does not receive unused numeric, byte-buffer, Unicode,
  collection, or object imports; adding one construct adds only its dependency
  closure.
- Source-policy tests identify the renderer as the only generated-import text
  producer.
- Equivalent matrices pass for all eight targets.
- Three complete generations are byte-identical.
- Every target formatter, linter, compiler, native test, public consumer, and
  sanitizer remains green.
- Uncached `//...` and `//:release_gate` pass in the Linux development
  container, including Rustfmt, Clippy, and Buildifier.

## Scope boundary

M30 requires compositional dependency ownership, not a universal grammar for
all target languages. A plugin may use a target-specific AST or the shared
`Document` algebra internally, provided every returned fragment is
dependency-complete and only its renderer can spell import directives.
