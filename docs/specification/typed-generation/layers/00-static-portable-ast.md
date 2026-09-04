# Layer 0: inferred typed portable AST

- Status: normative
- Input boundary: Rust source authored with typed constructors
- Output boundary: `TypedProgram<R>` with inferred requirements `R`

## Purpose

This layer makes the portable authoring surface valid by construction and
records exactly which portable functionality each program uses. A compiled
Rust generator may invoke a completed plugin `P` only when Rust can prove
`P: SupportsAll<R>` for that program's inferred requirements.

There is no monolithic or versioned feature profile. In particular, the public
architecture has no `StaticV1`. Unknown runtime input remains a separate,
fallible refinement boundary.

## Required public model

```rust
struct TypedProgram<R> { /* private checked program plus proof */ }
struct ProgramBuilder<R> { /* private accumulating builder */ }
struct Expr<T, R> { /* result type T; inferred requirements R */ }
struct Type<T, R> { /* value type T; representation requirements R */ }
struct Local<T, R> { /* callable-branded local */ }
struct Field<Record, T> { /* declaration-branded field */ }
struct Record<Record, Fields> { /* exact field list */ }
struct Function<Arguments, Result> { /* exact signature */ }

struct Nil;
struct Cons<Head, Tail>;
struct NoneRequired;
struct Requires<Feature, Tail>;
struct All<Left, Right>;

trait Supports<Feature> {
    type Mapping;
    fn mapping(&self) -> &Self::Mapping;
}
trait SupportsAll<Requirements> {}
```

Concrete names may differ, but the represented relationships and privacy
boundary are mandatory.

## Inference rules

- Every safe constructor returns a value whose requirement type contains the
  feature it uses and the requirements of all children.
- A consuming program builder returns a new builder type after adding a
  declaration. Therefore a caller cannot mutate the program while discarding
  the corresponding requirement proof.
- Requirement trees may contain repeated features. `SupportsAll` recursively
  checks every leaf, so deduplication is not required for soundness.
- Users normally bind the completed program with `let`; they do not manually
  name or maintain its inferred requirement type.
- No constructor accepts caller-provided evidence that a feature was used or
  supported.

The feature markers are independently implementable semantic families:

| Family | Markers |
| --- | --- |
| Declarations | `Functions`, `Records` |
| References | `LocalReads`, `FunctionCalls`, `RecordConstruction`, `FieldAccess` |
| Values | `BoolValues`, `I32Values`, `I64Values`, `F64Values`, `TextValues` |
| Operations | `BooleanLogic`, `Equality`, `Ordering`, `CheckedIntegerArithmetic`, `WrappingIntegerArithmetic`, `FloatingPointArithmetic`, `StringConcatenation` |

M34A-08U extends values with `CharValues`, `BytesValues`, `ListValues`,
`OptionValues`, and `ResultValues`, and extends operations with integer
bitwise/shift, float inspection, string inspection/transformation, bytes,
lists, options/results, numeric conversion, and UTF-8 conversion families.
Together these typed constructors cover every PolyIR v0 intrinsic. Interfaces,
control flow, and additional declarations remain separate branded AST work;
they are never smuggled through a generic intrinsic constructor.

Interfaces, interface values, implementations, composition, control flow, and
future constructs extend this catalogue with new independent markers rather
than a new profile version.

## Arbitrary typed lists

One recursive `Nil`/`Cons` representation is shared by:

- function parameter specifications;
- body-local handles derived from those parameters;
- call argument expressions;
- record field specifications and handles; and
- constructor value expressions.

A parameter list has an associated type list and a corresponding branded local
list. A function handle stores that exact type list. A call compiles only when
its argument list has the same type list. Records and constructors obey the
same rule.

The API MUST NOT expose `function0`, `function1`, `function2`, `callN`,
`recordN`, `constructN`, or an untyped public vector alternative. Recursive
lists make ordinary arity structural rather than an expanding API surface.

Target or resource limits are explicit checked constraints. They are not
encoded as arbitrary omissions from the authoring API.

## Construction invariants

- Public AST and proof fields are private.
- Expression operators encode operand and result types in Rust signatures.
- Function bodies return only expressions of the declared return type.
- Field projection requires a field and record value carrying the same
  invariant declaration brand.
- Local reads require a handle carrying the current callable-body brand.
- Record construction supplies every field exactly once and in declaration
  order through its typed list.
- Literal portable names pass constant lexical and protected-word checking.
- Final spelling, collision resolution, imports, and dependencies remain
  target-linker responsibilities.

## Target support

Every plugin registers individual executable mappings through a consuming
typestate builder:

```rust
let java = JavaPluginBuilder::new()
    .support(JavaFunctions)
    .support(JavaI32Values)
    .support(JavaWrappingIntegerArithmetic)
    .build();
```

Each call changes one `Missing` slot to `Implemented<M>` and requires
`M: FeatureMapping<JavaDialect, Feature = F>`. Only an implemented slot produces
`Supports<F>`, whose `mapping()` method returns that exact handler. Duplicate
registration is not representable, and a backend cannot manually write an
empty support claim.

Shared recursion derives the complete proof:

```rust
impl<D> SupportsAll<NoneRequired> for D {}

impl<D, F, Tail> SupportsAll<Requires<F, Tail>> for D
where
    D: Supports<F> + SupportsAll<Tail>,
{}

impl<D, Left, Right> SupportsAll<All<Left, Right>> for D
where
    D: SupportsAll<Left> + SupportsAll<Right>,
{}
```

There is no `SupportsAll<R>` fallback which treats unknown requirements as
supported. A backend method accepts `TypedProgram<R>` only with the explicit
bound `Plugin: SupportsAll<R>`.

## Static and dynamic convergence

Typed construction may replay through a private checked/CoreIR bridge during
migration. Safe callers cannot supply unchecked nodes to that bridge. Any
rejection is an implementation defect rather than a user diagnostic.

Unknown data follows:

```text
UnknownProgram -> checker -> Result<CheckedProgram, Diagnostics>
```

Downstream target ASTs and renderers do not distinguish the origin after a
valid program has crossed the appropriate proof boundary.

## Cross-language obligation

Rust, TypeScript, derived JavaScript, Python, Go, Java, C++20, and C17 all use
the same inferred portable requirements. Each independently declares support
for a feature only when its language specification provides total lowering,
certified target syntax, deterministic rendering, and native compiler/runtime
evidence for every admitted shape.

JavaScript inherits TypeScript's portable support proof and additionally
requires the pinned TypeScript derivation proof. It does not create an
independent semantic feature registry.

## Required proof

- Compile-pass a function and record containing at least three typed elements.
- Compile-fail mixed operands, wrong returns, wrong argument types/counts,
  wrong constructor types/counts, cross-record fields, cross-body locals,
  protected names, proof forgery, and missing feature support.
- Every exposed constructor replays through the checker and CoreIR verifier.
- An admitted Java program generates deterministically, compiles under the
  hermetic Java 21 toolchain with all warnings denied, and executes.
- Source policy rejects closed profiles, arity-numbered typed APIs, unchecked
  typed-program inputs, target support fallback implementations, and
  empty/manual backend support witnesses.
