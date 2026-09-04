# ADR-0008: Executable typed feature registration

- Status: accepted
- Milestone: M34A
- Date: 2026-09-04
- Amends: ADR-0004, ADR-0007

## Context

ADR-0007 made a typed program carry the exact portable features it uses, but
its original `Supports<F>` witness was empty. Java could therefore claim
support for `F` independently of the code which lowers `F`. The exhaustive
runtime capability registry and the typed admission proof could drift even
though both used closed enums.

Support is not a label. It is executable evidence that a plugin can map one
portable feature into its own typed target AST.

## Decision

There are two consuming builders with complementary proof obligations:

1. `ProgramBuilder<R>` infers the features a program requires and produces
   `TypedProgram<R>`.
2. `LanguagePluginBuilder<S>` registers typed mapping values and produces a
   plugin whose state `S` records exactly the features it implements.

Registration has the conceptual form:

```rust
LanguagePluginBuilder::new()
    .support::<Functions>(JavaFunctions)
    .support::<Records>(JavaRecords)
    .support::<BooleanLogic>(JavaBooleanLogic)
    .build()
```

`support::<F>(mapping)` is available only when the corresponding slot is
`Missing`, `mapping` implements `FeatureMapping<D, F>` for dialect `D`, and
its output is the target-AST category specified for `F` by `D`.

The call consumes the builder and changes that slot to
`Implemented<Mapping>`. A duplicate registration therefore does not compile.
The completed plugin implements `Supports<F>` only through that implemented
slot. `Supports<F>` exposes the registered mapping; it is never an empty marker
implementation and is never implemented manually by a backend.

Conceptually:

```rust
trait FeatureMapping<D, F> {
    type Input;
    type Output;

    fn lower(
        &self,
        context: &mut D::LoweringContext,
        input: Self::Input,
    ) -> Result<Self::Output, D::LoweringError>;
}

trait Supports<F> {
    type Mapping;
    fn mapping(&self) -> &Self::Mapping;
}
```

The concrete shared traits may split declaration, expression, statement, and
type mappings so inputs and outputs stay precise. A single erased input enum,
`Any`, reflection, strings, or `Box<dyn Any>` does not satisfy this decision.

The unknown-input capability preflight is derived from the same registration
catalogue. It may additionally reject a verified runtime shape, but it cannot
report a feature as native or emulated without a registered mapping. Typed
generation uses `Plugin: SupportsAll<R>`, not `Dialect: SupportsAll<R>`.

## Mapping granularity

A marker denotes the smallest independently implementable semantic family.
Closely coupled variants may share one exhaustive operation enum, such as
Boolean `not`/`and`/`or`. Unrelated string, numeric, collection, and tagged
operations do not share a catch-all support marker.

Every operation mapping receives already-lowered typed operands and returns a
typed target expression or expression plan. It cannot return source text,
tokens, imports, helper names, or unchecked target nodes. Target dependencies
are discovered later from returned AST references.

## Complete intrinsic surface

M34A-08U adds typed constructors for every intrinsic already defined by PolyIR
v0. They are grouped into independently registered families:

- Boolean logic, equality, and ordering;
- checked/wrapping arithmetic, bitwise operations, and checked shifts;
- floating-point arithmetic and inspection;
- string concatenation, inspection, slicing, replacement, trimming, and
  UTF-8-byte truncation;
- byte operations; list construction and operations; option/result
  construction and operations; numeric conversion; and checked UTF-8
  encode/decode.

The corresponding `Char`, `Bytes`, `List<T>`, `Option<T>`, and
`Result<Ok, Error>` typed value forms land in the same task because an
operation is not exposed before its operand/result types are representable.

## Consequences

- Removing a mapping removes the compile-time support proof.
- A support snapshot and executable dispatch cannot disagree about presence.
- Adding a feature extends the closed catalogue and plugin-builder slot set,
  but does not require every plugin to implement it.
- A language may remain incomplete: programs which do not use an absent
  feature still compile for it.
- Target verification and native compilation remain necessary: registration
  proves mapping presence and AST category, verification proves target syntax,
  and native tests prove semantics.

## Rejected alternatives

- Empty `Supports<F>` markers: claim and implementation drift.
- Runtime string maps: no compile-time absence or exhaustiveness proof.
- `Vec<Box<dyn Any>>`: erased input/output relationships.
- One all-features profile: unrelated missing features block valid programs.
- Renderer-owned semantics: bypasses AST verification and dependency derivation.

## Enforcement

- Source policy rejects empty/manual backend `Supports<F>` implementations.
- Compile-fail tests cover missing, duplicate, wrong-feature, wrong-dialect,
  and wrong-output registrations.
- Mutation tests remove one registration and prove typed admission or dynamic
  preflight fails.
- Every registered Java operation is invoked through its mapping object and
  generated output is compiled with Java 21 lint-as-error.

