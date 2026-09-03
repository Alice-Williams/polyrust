# Layer 3: capabilities and exhaustive registration

- Status: normative
- Inputs: verified `CoreProgram` and requested target set
- Outputs: one typed support decision per used feature and target

## Purpose

This layer lets a language support a portable feature natively, emulate it, or
reject only programs which use it. It preserves compile-time exhaustiveness
without making the compiler's build depend on every language implementing
every feature immediately.

For static programs, feature support is additionally a compile-time contract.
`StaticProgram<F>` may be passed to target `D` only when `D: Supports<F>`.
Runtime capability preflight remains mandatory for unknown/dynamic programs
and as a defensive migration assertion, but it is not the primary proof for
the static path.

## Feature model

Closed feature families and variants are Rust enums:

```rust
enum CoreFeature {
    Declaration(DeclarationFeature),
    Type(TypeFeature),
    Control(ControlFeature),
    Interface(InterfaceFeature),
    Operation(OperationFeature),
    Ownership(OwnershipFeature),
}

enum InterfaceFeature {
    Declaration,
    Conformance,
    MultipleConformance,
    StaticDispatch,
    DynamicDispatch,
    InterfaceValue,
}

enum SupportDecision<S> {
    Native(S),
    Emulated(S),
    Unsupported(UnsupportedReason),
}

enum UnsupportedReason {
    NotImplemented,
    Unrepresentable,
    UnsupportedShape,
    ToolchainUnavailable,
    ConflictingOptions,
}
```

`UnsupportedReason` is closed. Human detail belongs in a diagnostic field, not
in the discriminator.

## Feature use

A feature kind alone may be too coarse. `FeatureUse` carries the verified shape
needed for a support decision:

```rust
enum FeatureUse {
    Type(TypeUse),
    Operation(OperationUse),
    Interface(InterfaceUse),
    Control(ControlUse),
}

struct InterfaceUse {
    feature: InterfaceFeature,
    method_count: u32,
    has_interface_values: bool,
    parameter_types: Vec<CoreTypeId>,
    return_types: Vec<CoreTypeId>,
}
```

Shape data is derived from CoreIR and contains no target state. Equivalent uses
are canonicalized and deduplicated.

## Language capability registry

Every plugin owns one statically defined `CapabilityRegistry<D>`. The registry
maps each closed feature variant to a typed lowering strategy enum for that
dialect.

```rust
enum JavaInterfaceStrategy {
    NativeInterface,
    NativeInterfaceReference,
}

enum CInterfaceStrategy {
    StaticDirectCall,
    BorrowedFunctionTable,
    OwnedFunctionTable,
}
```

Built-in registries MUST be produced by exhaustive `match` or a macro which
generates an exhaustive `match`. Wildcard arms are forbidden. Adding a
`CoreFeature` variant must cause every built-in registry to fail compilation
until it explicitly acknowledges that variant.

Registries MUST NOT use:

- feature-name strings;
- helper-name strings;
- linker-section inventory;
- runtime reflection;
- discovery by scanning generated code;
- a default “supported” value; or
- a default “unsupported” wildcard.

## Mapping registration

A support decision and a lowering implementation are separate but linked.
`Native(strategy)` or `Emulated(strategy)` MUST name a strategy with a
registered lowering. An unreferenced lowering or strategy without a lowering
is an error.

A declarative macro SHOULD generate:

- the exhaustive support match;
- enum-keyed lowering dispatch;
- duplicate/missing registration checks;
- test-case enumeration; and
- diagnostic names.

Known operation mappings SHOULD expose statically typed functions. Dynamic
CoreIR values are additionally verified at generation time.

## Preflight

For each requested target, preflight:

1. enumerates exact `FeatureUse` values in CoreIR;
2. queries the plugin registry;
3. validates shape restrictions;
4. confirms every selected strategy has a lowering;
5. returns all unsupported-use diagnostics; and
6. invokes no lowering if any used feature is unsupported.

An unsupported feature in an unrequested target is irrelevant. When all eight
outputs are requested, every use must be native or emulated in all eight.

## Options

Target options are typed enums/structs owned by the plugin. A generic string
map may exist only at the CLI decoding boundary. Decoding must produce the
typed options before preflight.

An option may change a strategy only if the registry declares that choice.
Options MUST NOT add an unregistered feature or raw syntax.

## Diagnostics

An unsupported diagnostic includes:

- target ID and version;
- portable feature enum and use shape;
- closed reason enum;
- human detail;
- closest CoreIR provenance;
- available strategy alternatives when applicable; and
- the option which caused a conflict, when applicable.

Ordering is target, feature family, feature variant, source provenance.

## Required proof

- Compile-time test crates prove a new enum variant breaks incomplete built-in
  registries.
- Macro tests reject duplicate strategies, missing mappings, and wildcard
  fallback.
- Each plugin has an enumerated support snapshot.
- One-feature CoreIR fixtures exercise every support decision.
- Unsupported features reject only affected program-target pairs.
- All-target requests reject if any one target is unsupported.
- Repeated preflight returns byte-identical diagnostics.
- No capability or mapping dispatch compares string IDs.
