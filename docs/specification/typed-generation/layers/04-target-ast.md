# Layer 4: typed target AST

- Status: normative
- Input: verified `CoreProgram` plus a successful target preflight
- Output: opaque `VerifiedPackage<D>`

## Purpose

The target AST is the actual language-translation layer. It represents target
grammar and target representation decisions as typed Rust values rather than
source strings.

## Dialect ownership

Every independently lowered language owns:

- primitive and composite target type enums;
- expression enums;
- statement enums;
- pattern enums where applicable;
- declarations/items;
- modifiers and visibility;
- target-only heritage nodes where applicable;
- module/package/file syntax nodes;
- known symbol enums and catalogues;
- runtime helper enums and AST definitions;
- resolver policy;
- render-view types;
- template ID enums; and
- renderer.

JavaScript is compiler-derived from TypeScript and does not own an independent
semantic AST/lowerer.

## Grammar categories

Grammar categories MUST be different Rust types. For example:

```rust
enum JavaType { /* ... */ }
enum JavaExprNode { /* ... */ }
enum JavaStmt { /* ... */ }
enum JavaMember { /* ... */ }
enum JavaTopLevelItem { /* ... */ }
enum JavaFileItem { /* ... */ }
```

An API accepting `JavaExpr` cannot accept `JavaStmt`. An API accepting a
top-level item cannot accept an expression. Compile-fail tests cover every
category boundary.

## Typed expression builders

Known mappings SHOULD use phantom-typed handles:

```rust
struct Expr<D: TargetDialect, T: TargetTypeMarker> {
    id: TargetExprId,
    marker: PhantomData<(D, T)>,
}
```

Typed constructors enforce fixed signatures:

```rust
fn java_long_bit_and(
    left: Expr<JavaDialect, JavaLong>,
    right: Expr<JavaDialect, JavaLong>,
) -> Expr<JavaDialect, JavaLong>;
```

Heterogeneous AST storage erases the marker only through private arena APIs
which retain a concrete target type for verifier comparison.

Program-defined nominal types are dynamic. Their builders use
`TargetTypeId`/`TargetCallableId` and the target verifier checks their
signatures. Rust compile-time types cannot encode arbitrary declarations read
from `.poly.json`.

## Required node data

Each AST reference is one of:

```rust
enum TypeRef<D: TargetDialect> {
    Primitive(D::PrimitiveType),
    Known(D::KnownType),
    Generated(GeneratedTypeId),
    Runtime(D::RuntimeType),
    TypeParameter(TargetTypeParameterId),
    Constructed(D::ConstructedType),
}

enum CallableRef<D: TargetDialect> {
    Known(D::KnownCallable),
    Generated(GeneratedCallableId),
    Interface(GeneratedInterfaceMethodId),
    Runtime(D::RuntimeCallable),
}
```

The exact representation may be dialect-specific, but owner, origin, kind,
signature, and identity must remain recoverable without text parsing.

## Forbidden nodes

Production ASTs MUST NOT contain:

- raw/verbatim code;
- token streams;
- target snippets;
- interpolated templates;
- already rendered imports;
- source documents;
- arbitrary operator strings;
- arbitrary modifier strings; or
- arbitrary type/callable names.

Operators, modifiers, and declaration kinds are enums.

## Target-only constructs

A dialect AST may represent constructs absent from CoreIR, including a
target-only heritage clause, checked exception declaration, language attribute,
or ABI annotation. Such constructs:

- are created only by the language lowerer or structural runtime helpers;
- are typed dialect nodes;
- are validated under explicit target policy;
- cannot be requested by the generic frontend;
- cannot alter portable observable behavior; and
- require language-specific proof.

Certified generated inheritance is limited by the interface/composition
specification.

## Lowering

The lowerer traverses CoreIR exactly once for target syntax decisions. It:

- maps canonical types;
- selects target operations or helper references;
- allocates generated symbols;
- builds declarations and tests;
- preserves evaluation order;
- emits explicit clone/drop/control nodes where required; and
- records source provenance on produced nodes.

It MUST NOT:

- render target syntax;
- manually add imports/includes;
- compute helper closure;
- resolve name collisions;
- construct a manifest;
- scan a second semantic representation to repair dependencies; or
- approximate an unsupported feature.

## Unresolved AST verifier and certificate

Before resolution, the verifier checks:

- every arena reference exists and has the right category;
- expression and statement types agree;
- known callable arguments match compile-time catalogue signatures;
- generated callable arguments match declared signatures;
- receiver/static/constructor invocation kind is correct;
- scopes and local definitions are coherent;
- target grammar constraints hold;
- target-only constructs satisfy policy;
- all references remain symbolic rather than rendered;
- no forbidden opaque node exists; and
- all nodes retain provenance.

On success the verifier consumes `UnresolvedPackage<D>` and constructs
`VerifiedPackage<D>`. The verified wrapper's fields and constructor are private,
it has no deserialization implementation, and it exposes no mutable AST access.
Returning `Result<(), _>` while allowing the original package to continue is
not sufficient for the certified pipeline.

## Canonical dump

Every dialect MUST provide a deterministic, non-source AST debug dump. The dump
uses enum variant names and symbolic IDs and contains no rendered target code.
It is evidence for lowering determinism and architecture review.

## Required proof

- Compile-fail category and phantom-type tests.
- One positive and one negative test for every AST constructor.
- Forged-AST verifier failures for every invariant.
- Exact CoreIR-to-AST golden coverage for every used feature.
- No raw executable string accepted by any public or crate-visible builder.
- External compile-fail proof that `VerifiedPackage<D>` cannot be forged or
  mutated and `UnresolvedPackage<D>` cannot enter linking or rendering.
- Source policy rejects opaque node variants and render calls from lowering.
- Three repeated lowerings have identical AST dumps.
