# Layer 5: symbols, catalogues, and linking

- Status: normative
- Input: verified `UnresolvedPackage<D>`
- Output: verified `ResolvedPackage<D>`

## Purpose

This layer derives imports, includes, qualification, dependencies, helper
closure, names, and placement from typed references. It replaces manually
maintained import lists such as `require_java(body, "java.math.BigInteger")`.

## Symbol origins

Symbol origin is a closed enum:

```rust
enum SymbolOrigin<D: TargetDialect> {
    Primitive,
    LanguagePrelude(D::PreludeSymbol),
    StandardLibrary(D::StandardLibrary),
    ExternalPackage(D::PackageId),
    Generated(GeneratedSymbolId),
    Runtime(D::RuntimeSymbol),
    TypeParameter(TargetTypeParameterId),
    Local(TargetLocalId),
}
```

Origin controls eligibility for imports, qualification, package dependencies,
visibility, and file placement. Origin is never inferred from name text.

## Known symbol catalogue

Each plugin owns a closed catalogue of known types, callables, fields,
constructors, operators, annotations, and modules used by generated code.

A known callable specification contains:

```rust
struct KnownCallableSpec<D: TargetDialect> {
    owner: D::KnownType,
    name: D::KnownCallableName,
    origin: SymbolOrigin<D>,
    invocation: InvocationKind,
    type_parameters: &'static [TypeParameterSpec<D>],
    receiver: Option<TypePattern<D>>,
    parameters: &'static [TypePattern<D>],
    result: TypePattern<D>,
    failure: FailureBehavior,
    visibility: TargetVisibility,
}
```

`InvocationKind`, `FailureBehavior`, and visibility are enums. The AST stores a
known callable enum variant, not a separately supplied owner and method string.

A declarative definition SHOULD generate:

- all catalogue enums;
- metadata lookup matches;
- typed call constructors;
- symbol traversal;
- catalogue uniqueness checks; and
- documentation tables.

## Generated symbols

Program-defined declarations receive typed target IDs. The symbol table stores:

- original CoreIR ID and provenance;
- declaration kind;
- allocated target identifier;
- owning namespace and file group;
- type/callable signature;
- visibility;
- interface conformance; and
- optional target-only metadata.

Names are allocated deterministically before import resolution.

## Reference collection

The resolver traverses the typed target AST. Every node exposes typed symbol and
helper references through a dialect visitor. This is the authoritative
dependency discovery pass.

It is not legal to:

- attach imports manually while lowering;
- scan rendered code;
- search arbitrary strings for identifiers;
- rewalk CoreIR to infer imports; or
- add a fixed runtime import inventory.

## Name resolution

For each file, resolution determines:

- local and generated bindings;
- implicit/prelude names;
- required imports/includes;
- static/member imports when allowed;
- fully qualified references;
- aliases when the language permits them;
- deterministic generated renames;
- public re-exports;
- cross-file references; and
- external package dependencies.

Resolution MUST account for distinct type, value, label, macro, module, and
member namespaces where the language distinguishes them.

## Collision policy

Each language specification defines a deterministic preference order. The
shared requirements are:

1. never change a public portable name silently;
2. never shadow a local binding with an imported short name;
3. prefer qualification for known external symbols when aliasing is
   unavailable;
4. use stable suffixes only for private generated symbols;
5. produce a diagnostic if the language cannot represent the public collision;
   and
6. never choose based on hash-map iteration.

## Imports and includes

Import/include records are structured resolved data and can be constructed only
by the resolver.

They contain the semantic fields needed by the language, such as:

- module or header identity;
- imported symbol;
- alias;
- static/type-only/system/local classification;
- visibility/re-export status; and
- deterministic group/order.

Only the renderer spells a directive.

## Package dependencies

An external catalogue symbol may own a package dependency. Selecting that
symbol derives exactly one versioned dependency record. Conflicting version or
feature requirements fail resolution.

Standard-library and generated symbols do not create external package
dependencies.

## Helper linking

`HelperId` values are typed language enums or typed dynamic helper IDs. The
resolver:

1. collects roots from AST helper references;
2. resolves the deterministic transitive closure;
3. rejects missing helpers and cycles;
4. adds structural helper declarations;
5. repeats reference collection over selected helper AST;
6. places each helper once; and
7. derives all resulting imports and dependencies.

Helper names never determine closure.

## Resolved references

Every unresolved reference is replaced by a resolved reference which records
its final binding and spelling strategy:

```rust
enum ResolvedReference<D: TargetDialect> {
    Local(D::Identifier),
    Imported { binding: D::Identifier, import: ResolvedImportId },
    Qualified(D::QualifiedName),
    Member { owner: ResolvedTypeId, member: D::MemberName },
}
```

The renderer cannot change the variant.

## Resolved verifier

The verifier proves:

- no unresolved reference remains;
- every reference resolves to the correct namespace and signature;
- imports/includes exactly equal the reference-derived closure;
- package dependencies exactly equal selected external packages;
- every selected helper is present once;
- unselected helpers and dependencies are absent;
- visibility and cross-file access are legal;
- file dependency cycles satisfy target policy; and
- names are unique under target comparison rules.

## Required proof

- Catalogue completeness and uniqueness tests.
- Compile-time typed known-call tests.
- Collision matrices for every target namespace.
- Exact one-symbol import/include presence and absence tests.
- Import-free prelude symbol tests.
- External package derivation and version-conflict tests.
- Missing, duplicate, and cyclic helper diagnostics.
- Nested helper dependency tests.
- Forged resolved package rejection.
- Three identical link results from identical unresolved ASTs.
- Source policy rejects manual import APIs and text scans.
