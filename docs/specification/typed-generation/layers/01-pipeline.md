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
struct UncheckedDocument { /* serializable PolyIR */ }
struct CheckedProgram { /* private checker construction */ }
struct CoreProgram { /* private CoreIR lowerer construction */ }
struct UnresolvedPackage<D: TargetDialect> { /* target AST and symbols */ }
struct ResolvedPackage<D: TargetDialect> { /* linked target AST */ }
struct RenderView<D: TargetDialect> { /* private renderer construction */ }
struct RenderedPackage { /* rendered source and metadata */ }
struct OutputManifest { /* validated artifact tree */ }
```

Each type MUST own or immutably share its data. A phase MUST NOT mutate an
earlier representation to smuggle target state backward.

## Frontend contract

The verbose Rust builder remains the reference frontend. Future restricted
Rust parsers, macros, or other authoring forms MUST produce the same unchecked
PolyIR.

A frontend:

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

A target lowerer accepts only `CoreProgram` plus validated, typed target
options. It produces `UnresolvedPackage<D>`. It owns every target representation
choice but cannot render or assemble a manifest.

## Resolution contract

A target resolver accepts only `UnresolvedPackage<D>` and produces
`ResolvedPackage<D>`. It owns target names, imports/includes, package
dependencies, helper closure, declaration placement, and collisions. It cannot
change portable behavior.

## Rendering contract

A target renderer accepts only `ResolvedPackage<D>`. It constructs a private
typed `RenderView<D>` and applies that plugin's strict embedded Handlebars
templates. It cannot inspect CoreIR or choose target semantics.

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
       renderer -> handlebars adapter
           |
       manifest -> cli/materializer
```

Core crates MUST NOT depend on a concrete language plugin. A renderer MUST NOT
depend on the checker or CoreIR. The dependency-boundary gate enforces these
edges.

## Required proof

- Compile-fail tests reject each wrong phase input.
- Private-constructor tests prove later states cannot be forged.
- A fault-injection plugin cannot bypass linking or rendering.
- Failure injection at every phase emits no manifest and writes no file.
- Identical input/options produce identical phase dumps and manifest bytes.
- Dependency-boundary tests reject every forbidden crate edge.
- An external language plugin completes the same adapter pipeline.
