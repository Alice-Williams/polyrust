# ADR-0004: Typed target-AST generation pipeline

- Status: accepted
- Milestone: M34A
- Date: 2026-09-02
- Supersedes: ADR-0003 for all target-generation architecture
- Amended by: ADR-0005 for proof-carrying phase types and executable rendering

## Context

ADR-0003 made target fragments dependency-complete, but it allowed executable
syntax to remain opaque `Document`/`RawText` data with imports and helper roots
maintained beside it. That cannot prove that a referenced type or callable has
the right owner/signature, that dependencies were derived from actual
references, or that runtime bodies follow the same typed mapping rules.

The Java runtime example made the gap concrete: imports were structured only
after a handwritten list of qualified-name strings was attached to an opaque
runtime body.

## Decision

PolyRust adopts the normative
[typed-generation specification](../specification/typed-generation/README.md).
In particular:

- every frontend converges on unchecked PolyIR, which is checked and
  canonically lowered to verified target-neutral CoreIR;
- every independently lowered target owns a grammar-specific typed AST,
  exhaustive capability strategy, known-symbol/callable catalogue, structural
  helper catalogue, resolver, and renderer;
- known target behavior is selected with closed Rust enums and typed
  constructors; dynamic IDs identify only input-defined declarations and their
  text is never a semantic discriminator;
- imports/includes, qualification, helper closure, files, and package edges are
  derived from typed references by the resolver;
- unresolved, verified, linked, and render-ready packages are different Rust
  types and only an opaque render-ready package can reach source rendering;
- executable `Raw`, `Verbatim`, `Snippet`, source-string, and equivalent
  escape nodes are forbidden in production generation;
- as amended by ADR-0005, each plugin renders executable source with a total
  structural Rust formatter over a private render-ready package; executable
  Handlebars templates are forbidden and templates may serve metadata only;
- JavaScript executable source is the deliberate exception to an independent
  renderer: it is compiler-derived from the typed TypeScript package;
- portable PolyIR/CoreIR support flat interfaces, explicit conformance,
  first-class polymorphic values, and composition, but expose no inheritance;
  and
- a target AST may represent only the documented one-edge final framework
  adapter form of inheritance, with no portable or generated reuse hierarchy.

The plugin registry keeps an object-safe compiler adapter, while a safe plugin
API cannot bypass CoreIR verification, target verification, resolution,
rendering, or manifest assembly.

## Alternatives considered

- Retain dependency-complete text fragments: rejected because syntax and
  dependency metadata can still drift.
- Use one universal target AST: rejected because it admits invalid
  cross-language combinations and becomes either a lowest common denominator
  or a target-switch container.
- Use tokens/quasiquotation as the target IR: rejected as the primary boundary
  because tokens do not prove grammar category, callable signatures, ownership,
  or dependencies. A future convenience frontend may parse into the same AST.
- Use feature-sized Handlebars templates: rejected because it moves opaque
  semantic generation from Rust strings into template strings. ADR-0005
  extends this conclusion to every executable grammar template.
- Expose portable inheritance: rejected because flat interfaces plus explicit
  composition cover the intended portable behavior with clearer ownership and
  smaller test surfaces.

## Consequences

All current backends remain valid M30 dependency-fragment implementations but
are initially non-compliant with ADR-0004. They must migrate one at a time and
delete their old executable-string paths. The work is deliberately substantial:
each language needs grammar types, a symbol catalogue, structural runtimes,
resolution, a render-ready certificate, a total formatter, and focused
verification.

The existing semantic, native, differential, determinism, lint, sanitizer, and
historical-port tests remain mandatory regression evidence. M34-03 stays frozen
until all eight output paths pass the new contract. A future Rust parser can be
added without changing target plugins because it lowers into the existing
unchecked frontend boundary.

## Enforcement

The M34A tasks require compile-fail boundaries, exhaustive feature-registration
tests, AST and resolver fault injection, exact dependency/helper matrices,
certificate compile-fail tests, structural-renderer policy scans,
three-generation determinism,
language-native format/lint/compile/test gates, external consumers, C/C++
sanitizers, and replay of every completed real-world port.

The [typed-generation compliance ledger](../specification/typed-generation/compliance.md)
records migration evidence. A language passes only when its legacy raw path is
deleted and every requirement in its language specification is proven.
