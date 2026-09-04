# Portable capability catalogue

- Status: normative for M34A-08V and later language migrations
- Last updated: 2026-09-04

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
| `Loops` | bounded for-each, while, break, and continue |
| `PatternMatching` | exhaustive enum, option, result, boolean, and wildcard branching |
| `ResultPropagation` | explicit early propagation of portable results without exceptions |

Mutable locals, mutable records, exceptions, and unbounded target-specific
control flow are not part of the initial catalogue.

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

## Interface mapping bundle

The `Interfaces` mapping contains typed operations for interface and method
declarations, implementation declarations, exact method-to-function bindings,
conversion through a named conformance witness, concrete implementation calls,
and interface-value calls.

Generic interface identities, methods, records, and bindings are branded. An
implementation contains one binding for every declared method, no duplicates,
and no foreign method. Receiver, parameter, and result types are part of those
brands.

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
- Dynamic preflight derives support from the same registration and performs
  shape checks before invoking any mapping.
- Every accepted Java result passes post-link certification, total structural
  rendering, and hermetic Java 21 `-Xlint:all -Werror` compilation.
