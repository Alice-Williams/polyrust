# Layer 0: static portable AST

- Status: normative
- Input boundary: Rust source authored with typed constructors or macros
- Output boundary: `StaticProgram<F>`

## Purpose

This layer makes the initially supported portable language valid by
construction. If a Rust generator containing a static portable program
compiles, that program is well typed and can be generated for every selected
language whose type implements `Supports<F>`.

This is the primary authoring boundary. The unchecked document/checker path is
the separate boundary for data which is not known until runtime.

## Required public model

```rust
struct StaticProgram<F> { /* private typed portable AST */ }
struct Expr<T> { /* private expression node */ }
struct Type<T> { /* private type witness */ }
struct Local<T> { /* private symbol identity */ }
struct Field<R, T> { /* field of record R with value T */ }
struct Record<R, Fields> { /* exact constructor shape */ }
struct Function<Arguments, Result> { /* exact call signature */ }

trait Supports<F> {}
```

Closed portable primitive and feature sets use marker types or enums. Dynamic
declarations use opaque typed handles. Text is metadata or a checked spelling;
it is never symbol identity.

## Construction rules

- Public AST fields MUST be private.
- A public constructor MUST preserve every invariant expressed by its return
  type and MUST NOT accept an untyped expression or symbol ID.
- `Expr<T>` operators MUST encode operand and result relationships in their
  Rust signatures.
- A function body constructor MUST accept only `Expr<R>` for its declared
  return type `R`.
- A call MUST accept the exact argument tuple recorded by its function handle.
- A record constructor MUST accept every declared field exactly once with its
  declared type. Positional typed constructor handles MAY enforce this for a
  bounded initial arity.
- A field projection MUST pair a field handle with `Expr<Record<R>>` for the
  same `R`.
- Local and parameter expressions MUST originate from typed handles issued by
  their owning callable/scope builder.
- Target-neutral literal names MUST pass compile-time lexical and protected
  word checks. Final target spelling and collision resolution remain linker
  responsibilities.

## Feature profiles

Each static program carries a closed feature profile `F`. A target language
opts in explicitly:

```rust
impl Supports<StaticV1> for JavaDialect {}
```

Generation requires the bound:

```rust
fn generate<L, F>(program: &StaticProgram<F>) -> Source<L>
where
    L: Supports<F>;
```

There is no blanket implementation. Adding a profile or target therefore
forces an explicit compile-time support decision.

`StaticV1` contains only the constructs enumerated by ADR-0006. A type or
operation absent from that list has no public static constructor in this
profile.

## Static and dynamic convergence

Static construction may lower through a private compatibility bridge while
M34A migrates existing CoreIR. The bridge MUST NOT expose unchecked input and
MUST treat rejection as an internal invariant defect. Its output is immutable.

A future dynamic frontend performs fallible resolution and checking before it
may produce the same internal representation:

```rust
fn try_from_unknown(input: UnknownProgram)
    -> Result<DynamicallyCheckedProgram, Diagnostics>;
```

Downstream target code MUST NOT distinguish whether a valid program originated
from static construction or successful dynamic refinement.

## Java `StaticV1` mapping

Java declares `Supports<StaticV1>`. Its public static entry point accepts only
`StaticProgram<StaticV1>` and reuses the certified Java target pipeline. The
admitted profile maps to Java records, static methods, typed constructor calls,
field accessors, literals, and the already certified Java intrinsic mappings.

The entry point MUST NOT accept `CheckedProgram`, unchecked PolyIR, arbitrary
CoreIR, Java AST, or source strings under the static API name.

## Required proof

- Compile-pass: a typed record plus a two-argument mathematical function using
  nested grouping generates Java and executes successfully.
- Compile-fail: mixed arithmetic types, non-Boolean Boolean operations, wrong
  function return type, wrong call arguments, wrong record constructor fields,
  cross-record field access, protected identifiers, and a language lacking
  `Supports<StaticV1>`.
- Java output compiles with hermetic Java 21, `-Xlint:all`, and `-Werror`.
- Three identical generations produce byte-identical manifests.
- The static entry point cannot be called with an ordinary `CheckedProgram`.
- The full existing Java dynamic/certificate suite remains green.

