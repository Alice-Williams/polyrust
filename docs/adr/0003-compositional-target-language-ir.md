# ADR-0003: Dependency-complete compositional target-language IR

- Status: accepted
- Milestone: M30
- Date: 2026-09-01

## Context

M26 moved imports out of renderer-owned target switches, but several backends
still emit file-sized strings and then repair their dependencies with a second
walk or a fixed inventory. Such output may compile while silently accumulating
unused imports, missing imports, and target-specific semantic decisions in the
renderer. Runtime templates compound the problem by shipping every helper even
when the checked program needs only one feature.

## Decision

- Every target mapping produces a dependency-complete fragment containing its
  target syntax, direct structured imports, and direct runtime-helper roots.
- Composition is the only way fragments become closed file units; composition
  preserves all requirements associatively and deterministically.
- Import values contain validated semantic fields, never complete rendered
  directives. Only the language renderer spells dependency syntax.
- A backend may analyze checked semantics to select lowering or helper roots,
  but it must not repeat a syntax/type/value walk solely to repair imports.
- Runtime/support syntax is a stable helper graph. Deterministic transitive
  closure rejects missing nodes and cycles and emits each selected node once.
- Generated source roles use `LanguageSourceFile`; raw text files are limited
  to documentation, metadata, and explicit text assets.
- The complete normative rules and verification matrix live in
  [the target-language IR contract](../language-ir-architecture.md).

## Alternatives considered

- Keep file-sized source strings plus a final import scan: rejected because the
  syntax and its dependencies can drift and nested mappings cannot be verified
  independently.
- Let renderers inspect checked IR: rejected because renderers would become a
  second semantic translation layer and core generation would no longer be
  target-neutral.
- Keep monolithic all-feature runtimes: rejected unless a minimality test proves
  that every declaration and dependency is required whenever that one root is
  selected.
- Parse rendered target text to discover imports: rejected because it is a
  repair scan, conflates syntax parsing with semantic ownership, and cannot
  reliably identify helper dependencies.

## Consequences

Mappings become more explicit and independently testable. Adding a feature
requires dependency metadata at the point that its target syntax is introduced.
Backends may use different target ASTs, but all must preserve the same fragment
contract. Existing backends require migration and remain marked non-compliant
until exact positive/negative matrices and native gates pass.

## Enforcement

`LanguageFragment`, closed `LanguageUnit`, and `RuntimeHelperGraph` tests enforce
composition and closure semantics. `SourceFileRole`, `TextFileRole`, and private
`LanguageFile` variants prevent raw source-role bypasses; Bazel compile-fail
doctests prove the public boundary. The repository source-policy and deliberate
injection tests reject dependency directives outside renderers and exact native
fixture exceptions. The
[compliance ledger](../language-ir-compliance.md) may mark a surface `Pass` only
in the commit that adds its executable evidence.
