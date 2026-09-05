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
struct Block<T, R> { /* typed statements plus a result */ }
struct Statement<R> { /* typed statement with inferred requirements */ }
struct Field<Record, T> { /* declaration-branded field */ }
struct Record<Record, Fields> { /* exact field list */ }
struct Function<Arguments, Result> { /* exact signature */ }
struct Constant<T> { /* typed immutable declaration */ }
struct Alias<T> { /* transparent named type */ }
struct Enum<Variants> { /* payload-free variants */ }
struct Interface<Methods> { /* exact method signatures */ }
struct Implementation<Interface, Record, Bindings> { /* exact bindings */ }

struct Nil;
struct Cons<Head, Tail>;
struct NoneRequired;
struct Requires<Feature, Tail>;
struct All<Left, Right>;

trait Supports<Capability> {
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
- Requirement trees may contain repeated capabilities. `SupportsAll` recursively
  checks every leaf, so deduplication is not required for soundness.
- Users normally bind the completed program with `let`; they do not manually
  name or maintain its inferred requirement type.
- No constructor accepts caller-provided evidence that a capability was used or
  supported.

The closed capability set and ownership rules are defined by
`00-capability-catalogue.md`. The typed AST MUST expose the complete initial
catalogue through the following safe constructors:

| Capability family | Required typed surface |
| --- | --- |
| `Modules` | `typed_program` creates exactly one named module and always infers `Modules` |
| `Constants` | typed immutable declaration, constant expression, and same-module reference |
| `TypeAliases` | transparent named alias handle and named type witness |
| `Functions` | arbitrary parameter list, branded parameter read, exact call, and ordinary return |
| `Records` | declaration, exact construction, and branded field projection |
| `Enums` | non-empty payload-free declaration, variant value, enum equality, and exhaustive ordered branch list |
| `Interfaces` | arbitrary method signatures, exact implementation binding list, interface conversion, concrete call, and interface call |
| `PortableTests` | typed function or method invocation plus exact value/error expectation |
| `LocalBindings` | immutable binding which returns a fresh body-branded local handle |
| `Conditionals` | value-producing and statement `if/else` with equal branch result types |
| `Loops` | bounded `for_each`; its immutable iteration binding cannot escape the loop body |
| `PatternMatching` | exhaustive option, result, and boolean branches plus structurally constrained wildcard fallback |
| `ResultPropagation` | propagation of a fallible function or method call through its enclosing callable |
| value capabilities | typed type witnesses and constructors for unit, bool, i32, i64, f64, char, text, bytes, list, option, and result |
| operation capabilities | one named typed constructor per PolyIR operation, with exact operand and result markers |

No structural family is represented by a generic intrinsic constructor.
Interfaces, control flow, tests, constants, and aliases use branded types which
preserve their cross-reference relationships. Value capabilities own value
construction; operation capabilities remain independently selectable.

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
`M: CapabilityMapping<JavaDialect, Capability = C>`. Only an implemented slot produces
`Supports<C>`, whose `mapping()` method returns that exact handler. Duplicate
registration is not representable, and a backend cannot manually write an
empty support claim.

Shared recursion derives the complete proof:

```rust
impl<D> SupportsAll<NoneRequired> for D {}

impl<D, C, Tail> SupportsAll<Requires<C, Tail>> for D
where
    D: Supports<C> + SupportsAll<Tail>,
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
