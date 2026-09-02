# Typed generation specification

- Status: normative design baseline for M34A
- Implementation status: not yet compliant
- Accepted by: ADR-0004
- Supersedes: the target-generation portions of ADR-0003 and
  `language-ir-architecture.md`

This specification defines PolyRust's typed source-generation pipeline. The
words **MUST**, **MUST NOT**, **SHOULD**, and **MAY** are normative.

No current backend may claim compliance merely because it satisfies the older
dependency-fragment contract. Compliance requires all shared layer
specifications, the applicable language specification, and the permanent
evidence named by both.

## Specification map

Shared layers:

1. [Pipeline and phase ownership](layers/01-pipeline.md)
2. [Canonical CoreIR](layers/02-core-ir.md)
3. [Capabilities and exhaustive registration](layers/03-capabilities.md)
4. [Typed target AST](layers/04-target-ast.md)
5. [Symbols, catalogues, and linking](layers/05-symbols-and-linking.md)
6. [Interfaces, polymorphism, and composition](layers/06-interfaces-and-composition.md)
7. [Runtime helpers, files, and packages](layers/07-runtime-files-packages.md)
8. [Resolved render views and Handlebars](layers/08-rendering.md)
9. [Manifest assembly and verification](layers/09-manifest-verification.md)

Language specifications:

- [Rust](languages/rust.md)
- [TypeScript](languages/typescript.md)
- [JavaScript](languages/javascript.md)
- [Python](languages/python.md)
- [Go](languages/go.md)
- [Java](languages/java.md)
- [C++20](languages/cpp.md)
- [C17](languages/c.md)

Compliance:

- [Normative coverage cross-audit](coverage.md)
- [ADR-0004 migration ledger](compliance.md)

## Required phase graph

```text
Frontend
  -> Unchecked PolyIR
  -> CheckedProgram
  -> CoreProgram
  -> UnresolvedPackage<D>
  -> ResolvedPackage<D>
  -> RenderView<D>
  -> RenderedPackage
  -> OutputManifest
```

Every arrow is a typed, one-way boundary. Later phase constructors are private
to the phase that proves their invariants. A later phase MUST NOT recover
information by parsing or scanning an earlier phase's rendered text.

## Shared versus language-owned responsibility

| Concern | Owner |
| --- | --- |
| Portable syntax and source provenance | frontend and unchecked PolyIR |
| Portable legality and typing | checker |
| Canonical portable behavior and evaluation order | CoreIR lowerer/verifier |
| Feature use and support status | typed capability layer |
| Target syntax choices | language lowerer |
| Target grammar shape | language AST |
| Known target types/methods | language symbol catalogue |
| Imports, includes, qualification, collisions | language resolver |
| Runtime dependency closure | shared linker plus language helper catalogue |
| File layout and groups | language package policy |
| Syntax spelling | language renderer and templates |
| Output safety and declared artifacts | shared manifest assembler |
| Functional truth | evaluator, native tests, and pinned upstream oracle |

## Normative representation rule

Executable target code MUST be structured target AST until it enters the
renderer. There is no production `Raw`, `Verbatim`, `Snippet`, `Template`,
`Code(String)`, or equivalent AST node.

Strings before rendering may represent validated identifiers, literal values,
comments, documentation, paths, metadata, and diagnostic text. They MUST NOT
represent executable target syntax.

## Extensibility rule

A new output language implements a complete `LanguagePlugin` and its own
language specification. A new portable feature extends closed CoreIR feature
enums and forces every built-in plugin to acknowledge the feature as `Native`,
`Emulated`, or `Unsupported`.

Unsupported support in one language does not prevent other target plugins from
building or generating programs which do not request that language. The
all-eight compatibility track requires every used feature to be native or
emulated in all eight outputs.

## Interface and inheritance rule

Portable PolyIR and CoreIR expose interfaces, explicit conformance, static and
dynamic polymorphism, and composition. They expose no inheritance.

A target dialect may model a target-only inheritance construct for a framework
adapter or representation detail. Certified generation permits at most one
generated inheritance edge, forbids generated chains and reuse hierarchies,
and requires delegation to a composed component plus focused equivalence
evidence.

## Compliance labels

- **Not started**: no typed implementation exists.
- **Partial**: a useful layer exists but a required invariant or proof is
  missing.
- **Pass**: all required static, verifier, native, differential, determinism,
  and policy evidence is permanent and green.

Successful compilation alone is never a `Pass`.
