# ADR-0005: Intrinsic syntax validity and total source rendering

- Status: accepted
- Milestone: M34A
- Date: 2026-09-03
- Amends: ADR-0004 rendering and phase-boundary decisions

## Context

ADR-0004 removed raw executable source from target ASTs, but it still allowed
an independently executable Handlebars template to spell target grammar. A
strict template can reject missing data; it cannot make malformed punctuation,
misplaced keywords, or an invalid grammar skeleton unrepresentable in Rust.
Consequently, `ResolvedPackage<D>` was evidence that linking had succeeded, not
evidence that every renderer-accepted value must produce syntactically valid
source.

PolyRust needs a stronger and narrower initial theorem: every value accepted by
the certified renderer is syntactically valid for its declared language level.
This theorem does not yet claim functional equivalence, target type
correctness, or absence of target compiler bugs.

## Decision

Every independently rendered language uses the following one-way type-state
pipeline:

```text
CoreProgram
  -> UnresolvedPackage<D>
  -> VerifiedPackage<D>
  -> LinkedPackage<D>
  -> RenderReadyPackage<D>
  -> RenderedPackage
```

- `UnresolvedPackage<D>` is typed target AST which may have been assembled by
  fallible lowering and is not a rendering capability.
- The target-AST verifier alone constructs `VerifiedPackage<D>`.
- The resolver/linker consumes a verified package and constructs
  `LinkedPackage<D>`.
- A mandatory language-owned post-link checker validates whole-file grammar,
  names, declarations, symbol uses, target type rules required for syntax, and
  contextual restrictions. It alone constructs `RenderReadyPackage<D>`.
- All proof-carrying wrappers have private fields, no public constructors, no
  deserialization implementation, and expose immutable observations only.
- Verification and linking consume their input. There is no safe mutation path
  from a proved state back to its enclosed AST.
- The executable renderer accepts only `RenderReadyPackage<D>` and is a total,
  deterministic structural formatter. It owns fixed keywords, punctuation,
  delimiters, precedence parentheses, whitespace, and escaping in Rust code.
- The renderer has no error path for grammar selection and performs no
  validation, semantic decision, dependency discovery, or source parsing.
- Every grammar choice is an exhaustive match over closed target enums. Wildcard
  fallback arms and string-dispatched node kinds are forbidden.
- Executable Handlebars templates, token/string escape hatches, and third-party
  target AST/code-generation libraries are forbidden in the certified path.
  Strict templates may remain for non-executable metadata only when their bytes
  cannot be interpreted as source by a target compiler.
- JavaScript remains compiler-derived from the certified TypeScript package; it
  has no independent executable AST or renderer.

The guarantee is enforced by Rust visibility and ownership, not by convention:
safe callers cannot forge a render-ready certificate or call the renderer with
an earlier phase.

## Soundness boundary

The certificate means that PolyRust's language checker has proved every
documented syntactic and contextual invariant for the supported grammar
subset, and that the total formatter preserves those invariants. It does not
mean Rust's type system contains a formalization of an entire external language
standard.

Each language compiler/parser therefore remains an independent, pinned test
oracle. A deterministic generated corpus MUST establish the implication
`checker accepts -> renderer succeeds -> native compiler/parser accepts` for
the supported subset. Negative native fixtures MUST also be rejected by the
PolyRust checker. External formatters and linters test the theorem; they never
repair output or create the certificate.

## Alternatives considered

- Keep strict Handlebars for grammar spelling: rejected because template text
  remains an unchecked program outside the Rust grammar types.
- Treat successful native compilation as the certificate: rejected because it
  provides no compile-time API boundary and moves failure after rendering.
- Encode an entire evolving language grammar in Rust's type system: rejected as
  impractical. Opaque proof-carrying states plus an exhaustive checker provide
  the useful guarantee while admitting dynamic user declarations.
- Adopt a third-party AST generator: rejected for the core architecture because
  it would move PolyRust's soundness boundary into another API and dependency.

## Consequences

The M34A shared renderer and every language specification must migrate from
resolved-only Handlebars rendering to render-ready certificates and structural
formatters. Existing Handlebars work remains useful historical evidence for
escaping, precedence, and determinism, but is no longer sufficient for a
language `Pass`.

Java is the first implementation. Later language migrations use the identical
phase contract with their own target grammar and checker. Functional
equivalence remains mandatory for completed compatibility ports, but is not a
claim made solely by the syntax-validity certificate.

## Enforcement

- compile-fail tests prove earlier phase values cannot enter the renderer;
- external compile-fail tests prove clients cannot construct or mutate
  `RenderReadyPackage<D>`;
- source policy rejects executable templates, raw/token escape nodes, wildcard
  renderer matches, and string node dispatch;
- checker mutation tests cover every admitted grammar/context rule;
- accepted generated corpora compile or parse under the pinned native toolchain
  with warnings treated as errors where the language supports that mode;
- renderer calls are deterministic and cannot return a syntax diagnostic; and
- manifests are still assembled only after rendering by the shared adapter.
