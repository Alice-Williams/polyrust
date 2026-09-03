# Layer 1: pipeline and phase ownership

- Status: normative
- Input boundary: authoring frontend
- Output boundary: safe `OutputManifest`

## Purpose

This layer makes illegal phase crossings impossible or immediately
diagnosable. It defines which representation each phase accepts and produces,
which component owns every decision, and how failures remain atomic.

## Phase types

```rust
struct StaticProgram<F> { /* valid-by-construction portable AST */ }
struct UncheckedDocument { /* serializable PolyIR */ }
struct CheckedProgram { /* private checker construction */ }
struct CoreProgram { /* private CoreIR lowerer construction */ }
struct UnresolvedPackage<D: TargetDialect> { /* unproved target AST */ }
struct VerifiedPackage<D: TargetDialect> { /* locally valid target AST */ }
struct LinkedPackage<D: TargetDialect> { /* linked target AST */ }
struct RenderReadyPackage<D: TargetDialect> { /* whole-package certificate */ }
struct RenderedPackage { /* rendered source and metadata */ }
struct OutputManifest { /* validated artifact tree */ }
```

Each type MUST own or immutably share its data. A phase MUST NOT mutate an
earlier representation to smuggle target state backward.

## Frontend contract

The static typed Rust API is the reference frontend. It produces
`StaticProgram<F>` directly and follows Layer 0. The verbose legacy builder and
future parsers are dynamic frontends: they produce unchecked PolyIR and pass
through the checker.

A dynamic frontend:

- MAY allocate syntax `NodeId` values and attach `SourceRef` values;
- MUST emit only versioned portable syntax;
- MUST NOT resolve target names;
- MUST NOT construct checked or CoreIR nodes;
- MUST NOT select target capabilities, helpers, files, or templates; and
- MUST NOT call a backend.

The unchecked JSON reader remains bounded and rejects unknown schema fields.

## Checker contract

The checker is the only constructor of `CheckedProgram`. It owns:

- portable identifier and declaration resolution;
- portable type checking and assignability;
- interface conformance;
- method resolution;
- exhaustiveness and return coverage;
- constant legality and cycles;
- effect/capability legality; and
- stable source diagnostics.

It MUST collect all applicable diagnostics in deterministic order and MUST NOT
produce a partial checked program after an error.

## Core lowering contract

The CoreIR lowerer accepts only `CheckedProgram` and produces `CoreProgram`.
It removes portable authoring sugar, preserves behavior, and makes evaluation
order and temporary ownership explicit. It MUST run the CoreIR verifier before
returning.

## Plugin lowering contract

A static target entry point requires `D: Supports<F>`. For that admitted
profile, user-caused portable typing, capability, and target-syntax failures
MUST be unrepresentable; lowering is total. A temporary private bridge may
replay the dynamic checker as a defensive assertion during migration.

A target lowerer accepts only `CoreProgram` plus validated, typed target
options. It produces `UnresolvedPackage<D>`. It owns every target representation
choice but cannot verify, render, or assemble a manifest. The target-AST
verifier consumes that value and alone constructs `VerifiedPackage<D>`.

## Resolution contract

A target resolver accepts only `VerifiedPackage<D>` and produces
`LinkedPackage<D>`. It owns target names, imports/includes, package
dependencies, helper closure, declaration placement, and collisions. It cannot
change portable behavior.

## Render-readiness contract

A mandatory language-owned post-link checker consumes `LinkedPackage<D>` and
alone constructs `RenderReadyPackage<D>`. It validates the final compilation
units after helper composition, imports/includes, qualification, and file
placement. The wrapper is opaque outside shared certification code, cannot be
deserialized or safely mutated, and exposes only immutable observations.

## Rendering contract

A target renderer accepts only `RenderReadyPackage<D>`. It is a total
structural formatter whose exhaustive matches own target keywords,
punctuation, delimiters, precedence, escaping, and whitespace. It cannot fail
for a grammar decision, inspect CoreIR, choose semantics, discover
dependencies, parse source, or invoke an executable template.

## Manifest contract

The shared assembler accepts only `RenderedPackage`. It validates relative
paths, duplicate paths, roles, declared dependencies, helper reports, size
limits, and deterministic ordering before constructing `OutputManifest`.

## Plugin adapter

The public extension point is `LanguagePlugin`. The object-safe registry stores
a shared compiler adapter around a plugin. Plugins MUST NOT directly implement
an unrestricted `generate(CheckedProgram) -> OutputManifest` path.

```rust
pub trait LanguagePlugin: Send + Sync + 'static {
    type Dialect: TargetDialect;
    type Lowerer: TargetLowerer<Self::Dialect>;
    type Resolver: TargetResolver<Self::Dialect>;
    type Renderer: TargetRenderer<Self::Dialect>;

    fn descriptor(&self) -> PluginDescriptor;
    fn capability_registry(&self) -> &'static CapabilityRegistry<Self::Dialect>;
    fn lowerer(&self) -> Self::Lowerer;
    fn resolver(&self) -> Self::Resolver;
    fn renderer(&self) -> Self::Renderer;
}
```

## Failure and atomicity

Every phase returns a typed diagnostic set. A failure:

- MUST include the responsible phase and target when applicable;
- MUST retain the closest portable source provenance;
- MUST NOT produce a later-phase value;
- MUST NOT write a file;
- MUST NOT leave a partial manifest; and
- MUST be deterministic for identical inputs and options.

Filesystem materialization remains separate and atomic.

## Dependency direction

```text
ir <- check <- core
              ^
              |
target-ast <- language plugin
     |            |
     +-> linker <-+
           |
  post-link checker -> opaque render-ready package
           |
       total renderer
           |
       manifest -> cli/materializer
```

Core crates MUST NOT depend on a concrete language plugin. A renderer MUST NOT
depend on the checker or CoreIR. The dependency-boundary gate enforces these
edges.

## Required proof

- Compile-fail tests reject each wrong phase input, especially unresolved,
  verified, and merely linked packages passed to rendering.
- Private-constructor tests prove later states cannot be forged.
- A fault-injection plugin cannot bypass linking or rendering.
- Failure injection at every phase emits no manifest and writes no file.
- Identical input/options produce identical phase dumps and manifest bytes.
- Dependency-boundary tests reject every forbidden crate edge.
- An external language plugin completes the same adapter pipeline.
- A native compiler/parser corpus proves every checker-accepted generated case
  is accepted without formatter repair.
