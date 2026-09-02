# Layer 6: interfaces, polymorphism, and composition

- Status: normative
- Portable name: `Interface`
- Legacy v0 syntax name: `Contract`

## Purpose

Interfaces provide portable abstraction and dynamic polymorphism without
portable inheritance. Composition provides implementation reuse and object
assembly in every target language.

## Interface declaration

```rust
struct CoreInterface {
    id: CoreInterfaceId,
    name: PortableIdentifier,
    visibility: Visibility,
    documentation: Vec<DocumentationParagraph>,
    methods: Vec<CoreInterfaceMethod>,
}

struct CoreInterfaceMethod {
    id: CoreInterfaceMethodId,
    name: PortableIdentifier,
    documentation: Vec<DocumentationParagraph>,
    receiver: InterfaceReceiver,
    parameters: Vec<CoreParameter>,
    return_type: CoreTypeId,
}

enum InterfaceReceiver {
    Immutable,
}
```

An interface:

- is nominal in portable semantics;
- contains only required instance method signatures;
- contains no fields, constructors, static methods, default bodies, associated
  types, generic methods, or inherited interfaces;
- has unique method names because portable overloading is absent; and
- uses only portable parameter and result types.

## Implementation

One implementation links one generated record and one interface:

```rust
struct CoreImplementation {
    id: CoreImplementationId,
    record: CoreRecordId,
    interface: CoreInterfaceId,
    methods: Vec<CoreMethodImplementation>,
}
```

The checker requires exactly one body for every interface method, no extra
method, exact signatures, and no duplicate record/interface implementation.
A record may independently implement several interfaces.

Implementation bodies are ordinary pure CoreIR blocks with an immutable
`self` value.

## Interface values

`CoreType::Interface` is first-class. It may appear as:

- a parameter;
- a return value;
- a record field;
- an enum payload field;
- a list element;
- an option/result component;
- a local; and
- an intermediate expression.

Constants and canonical test values cannot directly encode an unknown dynamic
implementation. They construct a record and explicitly coerce it to an
interface value.

```rust
enum CoreInterfaceExpr {
    Coerce {
        implementation: CoreImplementationId,
        value: CoreExprId,
    },
    Call {
        interface: CoreInterfaceId,
        method: CoreInterfaceMethodId,
        receiver: CoreExprId,
        arguments: Vec<CoreExprId>,
    },
}
```

There is no implicit structural coercion and the checker does not rewrite an
ordinary expression. An authoring frontend must construct `Coerce` with a
specific implementation ID; the checker proves that the witness implements the
value's exact record type and determines the resulting nominal interface type.

Canonical portable-test values are the sole boundary exception because the
closed `Value` algebra deliberately has no interface-value or witness variant.
When a tested callable expects an interface, the checker accepts a record value
only if exactly one explicit implementation links that record/interface pair.
CoreIR and the evaluator resolve that already-proven pair deterministically;
this exception never applies to program expressions.

## Value semantics

An interface value owns an immutable value plus its implementation witness.
Copying or reusing it has the same structural value semantics as copying or
reusing the underlying record.

A target may borrow an interface value for one call, but it MUST NOT:

- expose pointer/reference identity;
- retain a borrowed receiver beyond the call;
- permit mutation through another alias;
- change dispatch after construction; or
- use garbage-collector identity as portable behavior.

Targets requiring manual ownership generate clone/drop support. Targets with
reference objects freeze or defensively copy the underlying value as needed.

## Dispatch

Static and dynamic calls are separate:

- `StaticMethodCall` names a known implementation and may lower directly.
- `InterfaceCall` receives an interface-typed value and dispatches through its
  implementation witness.

Argument evaluation is left-to-right and the receiver is evaluated exactly
once. Runtime failure uses portable `Result`/callable failure rules, never a
target exception as normal flow.

## Equality and inspection

Interface values do not support portable equality, ordering, hashing,
downcasting, reflection, dynamic type names, or implementation identity.
Those require later specifications.

## Composition

Portable composition is explicit record containment:

```text
record Service {
    renderer: Renderer
}

Service.render(input)
    -> self.renderer.render(input)
```

The composed field is an ordinary typed value. Delegation is an ordinary
explicit method call. PolyRust has no automatic member promotion, embedding,
base object, inherited lookup, `super` call, or override relation.

## Target-only inheritance

Inheritance is not exposed by unchecked PolyIR, checked PolyIR, CoreIR, or the
generic builder.

A target dialect may contain a typed heritage node for a framework adapter or
language representation. Certified target-only inheritance:

- adds at most one generated inheritance edge;
- may directly extend only an approved external/target-required base;
- cannot extend another generated subclass;
- cannot be the generated base of another subclass;
- forbids multiple inheritance, mixins, and interface-extension chains;
- is not used for implementation reuse or portable state;
- delegates implemented/overridden behavior to a composed component;
- is declared by a target enum, never a string; and
- is rejected before rendering if the heritage graph violates these rules.

An external base may internally have a deeper hierarchy. PolyRust adds only one
edge, does not inspect or depend on deeper implementation, and tests the
adapter contract directly.

Native conformance syntax such as Java `implements`, Rust `impl Trait`, Go
interface satisfaction, TypeScript `implements`, or Python protocol markers is
not portable implementation inheritance.

## Initial target strategies

| Target | Interface declaration/value strategy |
| --- | --- |
| Rust | trait, explicit impl, owned type-erased interface value with immutable borrow for calls |
| TypeScript | flat interface, explicit implementation, frozen tagged implementation witness |
| JavaScript | compiler-derived TypeScript object/witness representation |
| Python | flat `Protocol` typing plus frozen implementation witness |
| Go | flat interface plus compile-time assertion and value receiver |
| Java | flat interface plus explicit implementation and immutable reference/value wrapper |
| C++20 | composed type-erased value and function table by default |
| C17 | typed context/function table plus explicit clone/drop |

Each language specification refines the representation and proof.

## Legacy migration

Serialized IR 0.1 `contract` syntax remains readable only through the explicit
0.1-to-0.2 migration path. Canonical IR 0.2 serialization uses `interface`,
`interface_method`, and `interface` dispatch/reference fields; exact 0.2 readers
reject legacy spellings.

The IR 0.2 checker admits every first-class position listed above. During the
backend migration, a legacy backend that has not implemented interface values
must declare `FirstClassInterfaceValues` unsupported and reject the program
with an attributed capability diagnostic before translation. Its language task
removes that rejection only after its native ownership and composite-value
tests pass.

## Required proof

- Declaration/conformance positive and negative checker tests.
- Exact missing, extra, duplicate, receiver, parameter, and return diagnostics.
- Multiple independent interface conformance.
- Static and dynamic dispatch evaluator parity.
- Receiver and argument single-evaluation probes.
- Interface values in every admitted type position.
- Nested interface values through list/option/result/record/enum ownership.
- Independent composition and delegation tests.
- No portable inheritance node can be constructed or serialized.
- Target heritage verifier rejects every forbidden graph shape.
- Each target's native public consumer exercises dynamic dispatch.
- C/C++ clone/drop, failure injection, ASan, and UBSan coverage.
