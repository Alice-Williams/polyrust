# Layer 5: symbols, catalogues, and linking

- Status: normative
- Input: opaque `VerifiedPackage<D>`
- Output: `LinkedPackage<D>` awaiting whole-package certification

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

## Package-derived catalogue authority

`LinkerDialect::package_symbol_catalogue` derives an exact catalogue from the
checked unresolved `TargetAstPackage`. Its default delegates to the static
`symbol_catalogue`, preserving existing plugins. The hook is fallible: a
dependency cannot be resolved by falling back to an invented or incomplete
catalogue. It must be deterministic for identical immutable package authority.

The linker dialect value must equal the dialect retained by the original
package, not merely have the same Rust type. Both link and post-link boundaries
reject a mismatch before invoking any dialect hook. Stateful configurations
cannot validate under one policy and derive a catalogue under another.

The linker validates this derived catalogue before resolving references.
Post-link verification derives it again from the original unresolved package
and requires exact equality with the retained catalogue, including unused
entries and metadata. Internal consistency of modified linked names/imports
is insufficient. Certificate-backed dependency symbols remain language-owned;
this hook itself does not certify arbitrary caller-supplied metadata.

## Generated symbols

Certified generated dependencies are distinct from static known-library symbols
and consumer-owned generated declarations. `LinkerDialect` owns opaque
`DependencyCallable` and `DependencyPackage` types. A `DependencyCallableSpec`
records the exact callable, owner, native identifier, concrete typed signature,
typed spelling policy and provenance. `DependencySpelling` is a closed enum:
`FixedImport(ImportKind)` or `Qualified(QualifiedName)`.
Its language hook reconstructs every field from the
typed witness; catalogue validation requires exact agreement. A metadata record
alone never grants authority to generate a dependency call.

The shared reference is `TargetSymbolRef::DependencyCallable`, with
`SymbolOrigin::CertifiedDependency`. This linker category does not require a
second expression tree for compilation-unit dialects. Languages without this
mapping use an uninhabited associated type, not a sentinel or fallback.

`DependencyPolicy::FixedImport` requires the original native identifier. Both
initial resolution and post-link verification reject aliasing, including
reuse of a previously aliased physical import. These certified owners do not
invent package-manager versions or consumer source-file identities.

`DependencySpelling::Qualified` maps to the existing qualified reference policy
and retains the exact typed qualified name. It allocates no local import binding
or directive. The renderer receives that resolved name, not an alias suggestion.

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

Generated-file directives are distinct from symbol imports. A shared
`ResolvedFileImport<D>` retains the exact destination `TargetFileId` and a
dialect-owned import kind. Its construction is confined to the shared linker.
One real cross-file dependency yields at most one directive; it neither
allocates nor aliases the referenced generated symbols. The dialect's
`resolve_file_import` maps checked source/destination file facts to an import
kind or explicitly selects no directive (for example, Java same-package access).

The [typed file-requirement contract](../file-requirements.md) also permits a
structural dependency without a symbol. A dialect derives TargetFileRequirement
from typed AST/package metadata; shared linking authenticates the exact module
owner and output path, applies roles/cycles and deduplicates with symbol edges.
Unresolved requirements are not resolved-import authority.

Post-link verification reconstructs the ordered file-import list from actual
generated-symbol references, primary declaration placement and typed file
requirements, independently of the stored file-dependency list. Missing, extra, duplicate, retargeted and
kind-mutated witnesses fail. Existing visibility and cycle policies still apply.

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
3. rejects missing helpers and impossible definition/layout prerequisite cycles,
   while preserving legal callable reachability components under Layer 7;
4. adds structural helper declarations;
5. repeats reference collection over selected helper AST;
6. places each helper once; and
7. derives all resulting imports and dependencies.

Helper names never determine closure.

Dependency edge categories are explicit. Register the finite specialization
identities/prototypes before expanding callable bodies; a visited-identity
closure terminates legal callable cycles and emits each helper once. Complete
by-value layout prerequisites still require an acyclic order. Baseline runtime
helpers retain their direction rules: program-specific lifecycle/table
specializations are Implementation items, not runtime-to-user dependency escapes.

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

Binding verification reconstructs the complete ordered allocation from the
original checked AST and its reference-derived helper closure. Comparing only
linked names against other linked names is insufficient: coordinated changes to
bindings and references must not replace original declaration authority.

Fixed-import certified dependency names reserve the callable namespace across
the whole package, including unused registered witnesses. They must be distinct
from other fixed-import names and owned package-scope bindings under the
dialect's identifier comparison rules. Qualified dependencies instead collide
by their complete typed qualified name; their unqualified member names may
overlap each other, fixed imports and owned bindings. These checks run before
resolution and again during resolved verification; fixed native symbols cannot
be import-aliased. Neither policy weakens exact witness/catalogue reconstruction
from the original package, including unused registrations.

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
- Missing/duplicate and impossible prerequisite-cycle diagnostics; accepted
  finite callable components with deterministic once-only declaration placement.
- Nested helper dependency tests.
- Forged resolved package rejection.
- Three identical link results from identical unresolved ASTs.
- Source policy rejects manual import APIs and text scans.
