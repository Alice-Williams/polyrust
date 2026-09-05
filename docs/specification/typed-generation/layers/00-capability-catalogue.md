# Portable capability catalogue

- Status: normative for M34A-08V and later language migrations
- Last updated: 2026-09-05

## Purpose

Every construct admitted by the typed generic AST belongs to one closed,
target-independent semantic capability. A target plugin supports a capability
only by registering the complete typed mapping bundle for that capability.

A capability is not a target syntax node. `Interfaces` is a capability;
`JavaInterfaceDeclaration` is one representation selected by Java's mapping.
Universal validity rules are not optional capabilities: every typed program
always has valid identifiers, unique bindings, typed references, exact
arguments and returns, and immutable declaration identities.

## Granularity rule

A capability MUST be a coherent semantic contract. Operations which are
meaningless in isolation belong to the same capability:

- `Functions` owns declarations, parameters, calls, and ordinary returns.
- `Records` owns declarations, construction, and field projection.
- `Enums` owns declarations, variants, variant values, equality, and
  exhaustive enum branching.
- `Interfaces` owns declarations, method signatures, implementation bindings,
  interface values, concrete calls, and interface calls.

A capability MAY contain several lowering functions. Those functions form a
statically typed mapping bundle; they MUST NOT be stored as an erased callback
list. Capabilities remain separate where a target can coherently support one
without another or where they impose independently important semantics.
Checked and wrapping arithmetic are therefore separate.

## Initial catalogue

### Program and declaration capabilities

| Capability | Complete portable contract |
| --- | --- |
| `Modules` | named declaration groups, visibility, and typed cross-module references |
| `Constants` | typed immutable declarations, constant expressions, and references |
| `TypeAliases` | transparent named aliases with no independent runtime identity |
| `Functions` | arbitrary typed parameter lists, local parameter reads, calls, and returns |
| `Records` | immutable typed fields, exact construction, and field projection |
| `Enums` | finite named payload-free variants, variant values, equality, and exhaustive branching |
| `Interfaces` | method signatures, exact implementation bindings, interface values, concrete/interface calls, and multiple conformance |
| `PortableTests` | typed invocations and exact value/error expectations |

Portable enums have no variant payloads. Data associated with an enum is
represented by composition with records. The generic AST does not expose
classes, implementation inheritance, interface inheritance, default methods,
or target-specific enum bodies.

Composition requires no separate support claim: `Records + Interfaces`
expresses a component field and typed delegation to that component.

### Control-flow capabilities

| Capability | Complete portable contract |
| --- | --- |
| `LocalBindings` | immutable typed lexical bindings |
| `Conditionals` | statement and value-producing if/else |
| `Loops` | bounded immutable-list `for_each` iteration |
| `PatternMatching` | exhaustive option, result, boolean, and wildcard branching |
| `ResultPropagation` | explicit early propagation of portable results without exceptions |

Mutable locals, mutable records, exceptions, unbounded loops, `break`, and
`continue` are not part of the initial catalogue.

Payload-free enum branching belongs exclusively to `Enums`, because a target
cannot coherently claim the enum declaration/value contract without also
mapping every variant exhaustively. `PatternMatching` owns the remaining
portable sum-like and scalar patterns. Older untyped IR fixtures with
payload-bearing enum variants are compatibility input, not part of the typed
generic AST capability surface.

### Value capabilities

`UnitValues`, `BoolValues`, `I32Values`, `I64Values`, `F64Values`,
`CharValues`, `TextValues`, `BytesValues`, `ListValues`, `OptionValues`, and
`ResultValues` own their portable type, construction, and validity rules.

`CharValues` means Unicode scalar values. `BytesValues` and `ListValues` have
owned immutable value semantics. `OptionValues` and `ResultValues` never use a
target null or exception as their portable discriminant.

### Operation capabilities

- `BooleanLogic`
- `Equality`
- `Ordering`
- `CheckedIntegerArithmetic`
- `WrappingIntegerArithmetic`
- `IntegerBitwise`
- `CheckedIntegerShifts`
- `FloatingPointArithmetic`
- `FloatingPointInspection`
- `IntegerConversions`
- `StringConcatenation`
- `StringInspection`
- `StringTransformation`
- `Utf8Conversions`
- `BytesOperations`
- `ListOperations`
- `OptionOperations`
- `ResultOperations`

Construction belongs to the corresponding value capability. A target may
support storing a portable value without supporting every optional operation
over that value.

## Deferred extension catalogue

The closed catalogue can later add `UserDefinedGenerics`, `Maps`, `Sets`,
`Tuples`, `MutableLocals`, `MutableRecords`, `References`, `RawPointers`,
`ResourceManagement`, `Exceptions`, `AsyncFunctions`, `Threads`,
`FileSystem`, and `Networking`. Built-in `List<T>`, `Option<T>`, and
`Result<T, E>` do not require user-defined generics. A deferred capability is
absent, not silently approximated.

## Typed support contract

The shared API uses capability terminology:

```rust
trait Capability {
    type Index;
}

trait CapabilityMapping<Dialect>: 'static {
    type Capability: Capability;
    type Context<'a>;
    type Input<'a>;
    type Output;
    type Error;

    fn lower<'a>(
        &self,
        context: &mut Self::Context<'a>,
        input: Self::Input<'a>,
    ) -> Result<Self::Output, Self::Error>;
}

trait Supports<C: Capability> {
    type Dialect;
    type Mapping: CapabilityMapping<Self::Dialect, Capability = C>;

    fn mapping(&self) -> &Self::Mapping;
}
```

Associated input/context families MAY use an equivalent lifetime-safe design,
but mappings MUST accept portable typed AST or verified CoreIR information.
A mapping which accepts an already-complete target AST node and returns it
unchanged does not establish support.

Each `.support(mapping)` call consumes the plugin builder, infers
`mapping::Capability`, and replaces exactly one `Missing` slot with
`Implemented<Mapping>`. Only an implemented slot derives `Supports<C>`.
An exhaustive built-in registry must also call `.unsupported::<C>()` for every
capability it cannot yet implement. `Unsupported<C>` deliberately cannot derive
`Supports<C>`; adding a catalogue row therefore breaks incomplete registries at
compile time instead of silently claiming support. Catalogue rows are appended
so the type-level indices of existing capabilities remain stable.

## Interface mapping bundle

The `Interfaces` mapping contains typed operations for interface and method
declarations, implementation declarations, exact method-to-function bindings,
conversion through a named conformance witness, concrete implementation calls,
and interface-value calls.

Generic interface identities, methods, records, and bindings are branded. An
implementation contains one binding for every declared method, no duplicates,
and no foreign method. Receiver, parameter, and result types are part of those
brands. `InterfaceMethodList::Handles` and
`ImplementationBindingList::MethodHandles` are recursive associated types;
the typed implementation constructor requires them to be equal. Consequently,
missing, duplicate, or reordered bindings do not satisfy the constructor's
Rust trait bound.

The constructor issues a fresh `TypedImplementation` conformance witness and
fresh branded implementation-method handles. Interface conversion requires
that witness and a value carrying its exact record brand. Concrete dispatch
requires a method carrying the same implementation brand; interface dispatch
requires a method and receiver carrying the same interface brand. Declaring a
second implementation for the same record issues an independent witness and
is the portable representation of multiple conformance.

## Source layout

Shared markers and contracts live under
`crates/build/src/capabilities/`, with exactly one capability per snake-case
Rust file and a `mod.rs` containing only shared machinery and the closed
catalogue.

Each language owns a matching directory such as
`crates/backend-java/src/capabilities/`. It contains one mapping file per
supported capability. Registration order lives only in that directory's
`mod.rs`. Bazel targets use recursive globs; adding a capability file MUST NOT
require a hand-maintained source list.

## Proof obligations

- Adding a capability makes every exhaustive built-in registry fail until it
  records native, emulated, or unsupported status.
- Missing, duplicate, wrong-capability, wrong-dialect, erased-output, and
  target-source mappings fail during Rust compilation.
- Removing a mapping used by `TypedProgram<R>` makes its `SupportsAll<R>` call
  fail during Rust compilation.
- Every registered mapping has invocation evidence.
- A compile-pass inventory constructs at least one typed node owned by every
  initial capability and proves the resulting requirement tree is accepted by
  a complete test plugin.
- Removing the constructor-owned capability leaf for any inventory row makes
  that row's compile-time requirement assertion fail.
- Dynamic preflight derives support from the same registration and performs
  shape checks before invoking any mapping.
- Every accepted Java result passes post-link certification, total structural
  rendering, and hermetic Java 21 `-Xlint:all -Werror` compilation.
