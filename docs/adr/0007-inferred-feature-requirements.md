# ADR-0007: Inferred feature requirements

- Status: accepted
- Milestone: M34A
- Date: 2026-09-04
- Supersedes: ADR-0006 feature profiles and bounded-arity API
- Amends: ADR-0004, ADR-0005, and ADR-0006

## Context

`StaticV1` proved that Rust types can prevent invalid portable programs from
reaching a backend. It also coupled unrelated functionality into one profile
and exposed separate zero-, one-, and two-arity constructors. A target missing
one feature therefore could not accept a program which used none of that
feature, and extending ordinary function or record arity required new API.

Neither restriction belongs in the portable language. Backend admission must
depend on the exact functionality a program uses, and parameter, argument,
field, and constructor shapes must be recursively typed rather than bounded by
hand-written arities.

## Decision

The primary Rust frontend produces `TypedProgram<R>`, where `R` is an inferred
compile-time requirement tree. There is no versioned `StaticV1` profile.

- Every portable feature is represented by a sealed marker type.
- Every typed AST constructor contributes its marker to its result's
  requirement type.
- Expression requirements compose structurally; they need not be deduplicated
  for correctness.
- A consuming `ProgramBuilder<R>` accumulates the requirements of every added
  declaration. Ignoring a declaration's return value cannot omit its feature.
- Functions, calls, records, and constructors use recursively typed lists.
  Their lengths and element types must match at Rust compile time, with no
  public untyped-vector fallback.
- Each target implements `Supports<F>` separately for every feature `F` it
  implements completely.
- `SupportsAll<R>` recursively proves that a target supports every requirement
  carried by a program. It has no permissive blanket implementation.
- Target generation is callable only under `D: SupportsAll<R>`.

The static phase graph is:

```text
typed constructors
  -> ProgramBuilder<inferred requirements>
  -> TypedProgram<R>
  -> target D where D: SupportsAll<R>
  -> certified target AST
  -> RenderReadyPackage<D>
  -> OutputManifest
```

The unknown-input phase remains fallible and unchanged. Once unknown data has
been checked, runtime capability preflight remains its support proof.

## Feature catalogue

The initial typed builder distinguishes declaration and value features from
operations:

- functions, local reads, and function calls;
- records, record construction, and field access;
- Boolean, 32-bit integer, 64-bit integer, binary64, and text values;
- Boolean logic, equality, ordering, checked integer arithmetic, wrapping
  integer arithmetic, floating-point arithmetic, and string concatenation.

Interfaces, interface values, conformance, composition, control flow,
collections, options/results, and later functionality receive independent
markers. Adding one never renames or invalidates an existing feature set.

## Typed-list contract

`Nil` and `Cons<Head, Tail>` form the common structural list. Parameter lists
produce a corresponding list of branded locals. Function handles store the
parameter-type list. Call argument lists must have that exact type list.
Record declarations similarly produce branded field-handle lists, and record
constructors require an exact value list of the same shape.

This is structurally unbounded. Implementations may enforce an explicit
resource limit before target lowering, including a target's documented ABI or
class-file limit, but the public AST does not expose arbitrary `functionN` or
`recordN` families.

## Consequences

- Programs normally use inferred Rust types and do not spell `R`.
- Target support is granular: an unsupported feature rejects only programs
  which actually require it.
- Type signatures may contain repeated and deeply nested requirement nodes.
  Normalization is a compile-time performance optimization, not a soundness
  prerequisite.
- The existing checked/CoreIR bridge remains a defensive assertion until
  direct typed lowering replaces it. Rejection below an admitted typed path is
  a PolyRust defect.

## Enforcement

- Source policy rejects `StaticV1`, closed profile traits, and arity-numbered
  typed constructors from production code.
- Compile-fail tests prove wrong list lengths/types and missing target feature
  implementations cannot compile.
- Compile-pass tests use functions and records with at least three elements.
- Every accepted Java example is compiled with Java 21 and executed.
- All target specifications use the same `TypedProgram<R>` and
  `SupportsAll<R>` boundary even before their implementation migration begins.
