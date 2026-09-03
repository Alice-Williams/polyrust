# Layer 2: canonical CoreIR

- Status: normative
- Input: `CheckedProgram`
- Output: verified `CoreProgram`

## Purpose

CoreIR is the canonical erased semantic form shared by language plugins. It
prevents eight plugins from independently desugaring the same portable
construct and makes behavior suitable for direct evaluator comparison.

The static generic AST precedes CoreIR and carries relationships such as
`Expr<T>`, typed symbol ownership, callable signatures, and feature profile in
Rust types. Erasure into CoreIR MUST preserve those relationships. During
migration a defensive verifier may recheck them; its rejection is an internal
defect for a `StaticProgram<F>`, not an expected static-authoring outcome.

CoreIR remains structured source-generation IR. It is not machine code, SSA,
LLVM IR, or target syntax.

## Identity and type model

Closed semantic sets use enums. Dynamic declarations use newtyped arena IDs.

```rust
enum CoreType {
    Unit,
    Bool,
    I32,
    I64,
    F64,
    Char,
    String,
    Bytes,
    List(CoreTypeId),
    Option(CoreTypeId),
    Result { ok: CoreTypeId, error: CoreTypeId },
    Record(CoreRecordId),
    Enum(CoreEnumId),
    Interface(CoreInterfaceId),
}

struct CoreExprId(u32);
struct CoreBlockId(u32);
struct CoreFunctionId(u32);
struct CoreInterfaceId(u32);
struct CoreInterfaceMethodId(u32);
```

IDs identify arena entries only. Production behavior MUST NOT branch on an ID's
numeric or textual representation.

## Program shape

```rust
struct CoreProgram {
    module: CoreModule,
    types: TypeArena,
    expressions: ExpressionArena,
    blocks: BlockArena,
    provenance: ProvenanceMap,
    features: FeatureUseSet,
}

enum CoreDeclaration {
    Constant(CoreConstant),
    Alias(CoreAlias),
    Record(CoreRecord),
    Enum(CoreEnum),
    Interface(CoreInterface),
    Implementation(CoreImplementation),
    Function(CoreFunction),
    Test(CoreTest),
}
```

Declaration and member order is canonical wherever ordering is non-semantic.

## Expressions and statements

Each operation has its own enum variant with fields appropriate to that
operation. There is no operation name string plus untyped argument vector in
CoreIR.

```rust
enum CoreExpr {
    Local(CoreLocalId),
    Constant(CoreConstantId),
    Literal(CoreLiteral),
    ConstructRecord { record: CoreRecordId, fields: Vec<CoreFieldValue> },
    ConstructEnum { variant: CoreVariantId, fields: Vec<CoreFieldValue> },
    Field { value: CoreExprId, field: CoreFieldId },
    Call { function: CoreFunctionId, arguments: Vec<CoreExprId> },
    StaticMethodCall { implementation: CoreImplementationId, method: CoreInterfaceMethodId, receiver: CoreExprId, arguments: Vec<CoreExprId> },
    InterfaceCall { interface: CoreInterfaceId, method: CoreInterfaceMethodId, receiver: CoreExprId, arguments: Vec<CoreExprId> },
    Intrinsic(CoreIntrinsicExpr),
    If { condition: CoreExprId, then_block: CoreBlockId, else_block: CoreBlockId },
    Match(CoreMatchExpr),
    Block(CoreBlockId),
}

enum CoreStatement {
    Let { local: CoreLocalId, ty: CoreTypeId, value: CoreExprId },
    ForEach { binding: CoreLocalId, iterable: CoreExprId, body: CoreBlockId },
    Return(Option<CoreExprId>),
    Evaluate(CoreExprId),
}
```

`CoreIntrinsicExpr` is a closed enum family. Variants carry named operands
rather than an untyped vector wherever practical.

## Evaluation order

CoreIR evaluation is strictly left-to-right. Core lowering introduces
temporaries when:

- an input expression would otherwise be evaluated more than once;
- an allocation or failure could move relative to another operation;
- target argument order is not guaranteed;
- cleanup must observe a partially initialized state; or
- interface receiver evaluation could be duplicated.

A plugin MUST preserve CoreIR order and MUST NOT re-run authoring-level
desugaring.

## Values and ownership

CoreIR retains immutable structural value semantics. It marks:

- borrowed call inputs;
- owned results;
- clone points required by semantic reuse;
- destruction scopes for fallible lowerings; and
- interface-value ownership.

These marks describe portable behavior, not a target pointer/reference syntax.

## Interfaces

`CoreType::Interface` is a first-class type. Interface declarations contain
flat method signatures; they do not extend other interfaces. An implementation
links one record and one interface with explicit method bodies.

Static implementation calls and interface-value dynamic calls are distinct
expression variants. This distinction is observable in capability use but not
in method results.

## Composition

Composition is ordinary record containment plus explicit delegation calls.
CoreIR has no base-class field, inherited member lookup, `super` expression,
override relation, or heritage graph.

## Lowering from checked PolyIR

The lowerer MUST:

- resolve aliases to canonical types while preserving public alias declarations
  when required for API generation;
- replace contract terminology with canonical interface nodes;
- normalize method dispatch into static or interface calls;
- make fallible operations explicit;
- make evaluation order explicit;
- retain exact binary64 bit literals;
- canonicalize pattern and declaration order;
- retain source provenance; and
- compute exact feature uses from the produced nodes.

It MUST NOT choose a target strategy or helper.

## Verifier

The CoreIR verifier checks:

- all arena IDs exist and have the expected category;
- each expression's stored type equals the type derived from its variant;
- calls have exact receiver, argument, and return types;
- interface methods belong to the referenced interface;
- implementations are complete and signature-exact;
- blocks have coherent result and return types;
- match arms are exhaustive and non-duplicated;
- temporary definitions dominate uses;
- ownership/clone/drop markers are balanced;
- feature use equals the exact node-derived set;
- no inheritance or target data exists; and
- provenance exists for every semantic node.

Verification returns all deterministic diagnostics and no `CoreProgram` on
failure.

## Serialization and dumps

CoreIR is an internal representation and is not the public `.poly.json` schema.
It SHOULD support a canonical debug dump used for golden tests and
determinism. Debug dumps MUST contain stable IDs and no addresses, timestamps,
absolute paths, or hash-map order.

## Required proof

- Every checked syntax form has a CoreIR golden.
- Equivalent authoring forms lower to identical CoreIR.
- Invalid forged CoreIR fails each verifier invariant.
- Evaluator results before and after CoreIR introduction are identical for the
  canonical conformance corpus and every historical port.
- Evaluation-order probes prove single evaluation and failure ordering.
- Interface static/dynamic dispatch and explicit delegation have evaluator
  vectors.
- Parse/check/lower repeated three times yields byte-identical CoreIR dumps.
