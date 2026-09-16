# Rust public scalar constants in C17

- Status: owned and imported C target mappings complete in M35-03A-02F-02B-02; owned source/bundle publication implemented; foreign source integration pending
- Parent: [shared source contract](../../rust-public-constants.md)
- Reuse: [C package projection](rust-hir-public-packages.md)

## Owned C declaration and definition

Reuse CObjectRef, CObjectType, CScalarType, CConstness, CDeclarations,
CInitializer, CLiteral and CSignedLiteral. Register the public object against
its GeneratedPublicHeader with exact bool/i32/i64 scalar type and top-level
const qualification. Emit one extern object declaration in the header and one
external-linkage literal-initialized definition in its GeneratedSource.

The existing registered file identity ties declaration and definition together;
never duplicate an object identity per file. Reject missing/duplicate definitions,
wrong file/owner/linkage/type/qualifier/initializer, or an initializer outside
the bounded scalar constant profile. Values use the existing exact minimum
spelling and platform obligations. Type references derive standard includes;
the shared linker derives the implementation's header import.

This is an ordinary readonly C object API, not a C integer constant expression.
It is not promised usable in case labels or constant array bounds. No macro
substitution, inline accessor function, custom runtime, static initialization
helper, or string-based declaration is permitted.

## Source certification and public reads

Extend the bounded Rust-source package/effect profile to accept precisely these
immutable scalar objects and their typed reads. Generic mutable/global storage
does not become admissible. Source registration and documentation projection
retain RustSource declaration origins and effective public visibility.

Owned reads use CExpressions::global followed by the existing typed read, with
constness represented on the place and the correct scalar value type. No source
borrow/address operation is enabled by this change. Resource accounting includes
every object/header declaration/definition/reference and their actual formatted
bytes; immutable object reads do not invent runtime call frames.

Constants-only packages retain normal header/source files, checked guards,
namespace/layout obligations and complete source exports without synthetic
functions. Existing function and record safety checks still run when present.

## Dependency boundary

Child02B-01 replaces the function-only producer API with a complete certified
function-and-constant inventory. Read-only CDefinedConstant views and opaque
CDependencyConstant witnesses retain the actual registered const object, literal,
unqualified read type, source identity, allocated symbol and owning files.
Constants-only producers have zero function frames, not a fabricated function.
Synthesized constants cannot stand in for Rust public declaration provenance.
Owned consumers check their resolved names against every direct and transitive
dependency export, whether or not that export is referenced. Child02B-02 completes
authenticated consumer constant registration and reads with native proofs.

CDependencyConstant is derived only from a certified producer; the consumer
mapping binds a CImportedValue through CRegistry. Retain producer package,
Rust declaration origin, exact const scalar type/value, fixed symbol and header
identity. Foreign objects cannot be registered as owned objects or authenticated
from a supplied name, JSON, raw signature or copied RustSource metadata.

Extend the shared typed dependency-value catalogue using the same certified
package import kind as functions. DependencyValueSpec.ty uses the constant's
unqualified scalar read type.
The const-qualified storage type remains on CObjectRef and the opaque constant
witness; it must not be erased there or encoded as a constructed dependency
value type. Derive the read type through the existing C value-conversion rules.
The complete producer public-symbol inventory
includes objects and functions, including unreferenced exports. Check all C
ordinary-identifier collisions, not only referenced function names. Imports and
resource bounds cover values-only and mixed dependencies.

Imported objects live in a separate registry witness map, never the owned
declaration/definition inventory. General reads authenticate the consumer brand
and original object metadata; declaration builders require owned references.
Values and functions share producer identity and header/symbol conflict checks
in either registration order. Unused registrations retain complete dependency
closure obligations without producing unnecessary includes.

The memory checker seeds initialized foreign scalar roots from retained producer
evidence. Numeric reads use the exact immutable certified literal through the
existing numeric-literal evaluator, not a caller-supplied range. Address borrowing
remains outside this source profile. Exact numeric evidence does not enable
otherwise unsupported arithmetic source admission.

Dependency traversal follows both value and function witnesses. A constants-only
producer may have a measured zero-frame bound; equality to original measurements,
all resource limits, and actual callable costs remain mandatory.

## Proof

Use registered typed packages to test constants-only and mixed headers/sources,
exact values, independent header inclusion, separate GCC/Zig O0/O2 compilation
and strict warning flags. Native assignment through each public const object
must fail. Invalid qualifiers/definitions/owners/linkage/initializers and forged
or stale imported witnesses must fail the appropriate typed/certificate boundary.

Compiler and bundle integration subsequently prove aliases, docs, source-only
private constants, exact multi-crate values and atomic publication. This target
step alone does not claim complete Rust-source public-constant support.

## Compiler assembly prerequisite

Follow the C section of [package/function state](../../rust-source-package-state.md).
Package registration owns the registry, source file, declaration inventories and
origin cache independently of function analysis. Body Readers are constructed
only for registered functions; zero bodies require no TypeckResults or dummy
function. The complete C target checks still run after file assembly. This
structural prerequisite does not itself admit source constant APIs.

## Single-crate compiler publication

PublicConstants maps a private compiler-evaluated input into package-state
registration and returns its CObjectRef. PublicConstantReads resolves the exact
owned object by DefId and checks its Rust origin, header and evaluated value;
the generated read cannot silently become a folded literal. Both mappings are
required consuming-builder slots, with package-State and function-Reader
contexts respectively.

Single-crate api.json retains schema 1 for function-only packages and uses schema
3 when constants are present. Module bindings distinguish constant/function.
Each constant records its exact identity, allocated symbol, primary header,
implementation source, scalar type, readonly flag and value. Bool values are JSON
booleans; i32/i64 values are canonical signed decimal strings, preserving values
outside double precision. Metadata is reconstructed against certified objects
and the compiler-owned expected inventory in both directions. Constant-bearing bundle
owners use schema 4 below; schema 2 remains the function-only bundle format.

The manifest accepts only RenderReadyPackage<CDialect>. C profile certification
already requires public-header const bool/i32/i64 objects with exact matching
literal initializers; schema 3's readonly/type descriptions depend on that
certificate, not on unverified caller-supplied ASTs. Manifest reconstruction
separately checks compiler identities, export completeness, files, linkage,
values and allocated names. Repeating the profile's constness/type checker in
the JSON layer is not a substitute for preserving this certificate boundary.

## Owned constant bundle schema

Child05A extends owner manifests with schema 4 for bundles containing owned
constants. Its function/import records keep their existing structure, and
constant records match standalone schema 3's exact identity, symbol, owner files,
scalar type, readonly flag and lossless value representation. Owners without
constants retain schema 2. The outer bundle index remains schema 1 because its
member references are unchanged. Readers must dispatch explicitly by version.

Whole-bundle preflight reconciles each manifest with its retained original
CDependencyApi and counts external constant definitions in the same symbol set
as external functions. All output names and source/metadata bytes are reserved
before publication. Constants-only owners, mixed owners and declared unused
dependency owners are retained. This step does not authorize foreign constant
body reads or foreign public re-exports; those require the subsequent compiler
join and export-closure checkpoints.
