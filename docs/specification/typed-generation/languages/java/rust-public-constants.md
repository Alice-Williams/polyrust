# Rust public scalar constants in Java 21

- Status: normative; owned source/bundle publication implemented; foreign source integration pending
- Parent: [shared source contract](../../rust-public-constants.md)
- Reuse: [Java source packages](rust-hir-lowering.md)

## Owned Java field

Reuse GeneratedValueId/GeneratedValue, JavaField, JavaMember::Field,
JavaModifier, JavaType and JavaLiteral. Register each constant with exact
RustSource declaration provenance and primitive boolean/int/long type. Place
one public static final field with an exact typed literal initializer on the
crate facade. Allocate protected field names through the existing typed name
and linker path mechanisms.

Generated registration, containing class, field declaration, visibility,
static/final modifiers, primitive type and evaluated initializer must agree.
Do not treat any arbitrary static field as a certified source constant.
Missing/duplicate fields, wrong class/owner, mutable/nonstatic fields, widened
or narrowed values and nonliteral initialization fail the bounded source profile.

Constants-only facades retain the ordinary private facade constructor and
module documentation but synthesize no dummy source method. No Runtime class,
boxing, accessor helper, static initializer block, raw Java or copied runtime
file is added.

## Reads, source descriptions and resources

Owned reads use the existing generated-value reference representation and exact
GeneratedValueId. Preserve primitive TypePlan and field declaration ownership.
Java compiler constant folding is allowed; generator references still retain
typed dependency identity until rendering. Constant storage borrowing and
mutable-field operations are not enabled.

Extend source inventory/description and bounded dependency-package verification
to recognize registered scalar constant fields alongside source methods. Verify
export completeness, finite alias bindings, doc routing and declaring facade
paths. Charge field syntax, names, literal spelling, references and qualified
paths to the current source/resource limits. Fields add no runtime call frames.

## Dependency boundary

Add opaque JavaDependencyConstant and JavaImportedValue witnesses, constructible
only from independently certified producer packages. Add a distinct dependency
value variant to JavaValueRef and its typed expression projection. It retains
producer identity, source declaration, primitive type/value and exact declared
field path. Never substitute KnownField, a synthetic callable or a free-form
qualified-name string for producer authority.

Use the shared dependency-value catalogue. Render the witness-derived qualified
field path; no import string list or guessed local alias. Preserve complete
producer exports, including public constants not selected by a consumer.
Reconstruct all reference/spec/owner metadata from original package authority
during post-link verification.

## Proof

Build typed constants-only/mixed facades and inspect actual registration/field
and read nodes. Strict Java 21 compilation and independent execution cover both
bools, exact signed boundaries, wide integers and values shared across separately
compiled producer/consumer classes. Assignments to exported fields must fail
native compilation. Type/value/finality/owner/path/export and linked-reference
tampering must fail structural checks or independent native truth as appropriate.

Final compiler/bundle work adds real Rust-source constants, aliases, private
names, docs, stale producer metadata and atomic publication. Existing Java
certification, source-function and private/local constant tests stay enabled.

## Producer implementation boundary

JavaDependencyApi reconciles the complete public function/constant union.
JavaDependencyConstant retains the defining RenderReadyPackage through its
JavaDependencyPackage, the generated value ID, Rust source declaration, declared
field path, primitive type and exact literal. Equality/ordering distinguish
independently certified owners even when source IDs and emitted text coincide.
Borrowed descriptions have a distinct Constant case; descriptions and resolved
items cannot be promoted into constant authority.

The original render-ready certificate authenticates the source inventory against
the original registration table. The narrower producer projection additionally
checks literal/type/static/final/public/owner/export agreement and admits method
reads only from its verified constant inventory. Neither layer claims that
caller-constructed source metadata authenticates a real Rust program: that join
belongs to the rustc adapter and the compiler/bundle milestones.

Producer tests use separately compiled Java21 consumers. Since javac can inline
constant-variable reads, semantic-mutant tests recompile consumers against each
mutant producer; running stale consumer bytecode is not sufficient evidence.
The Java bundle projection/serialization support Constant descriptions through
the owned bundle schema below. Foreign compiler reads/re-exports remain separate
integration checkpoints. Existing source kinds stay enabled.

## Consumer implementation boundary

JavaDependencyScope::import_constant consumes a JavaDependencyConstant and
returns a JavaImportedValue branded to that scope. Freezing preserves both value
and function registrations; the same exact witness may be registered repeatedly
without creating distinct bindings. No raw name/path or unchecked package can
construct an imported value. Expressions carry JavaValueRef::Dependency, which
must have the defining primitive type and appear in the containing file's exact
frozen scope.

Shared DependencyValueSpec entries derive names, owner identity, primitive type
and qualified spelling from those opaque witnesses. Post-link checks rederive
the original package and compare each dependency spelling with its exact producer
path. The renderer merely emits that checked reference. Assignment remains
forbidden, and no foreign field is promoted to an owned declaration.

Dependency authority retains all registered producer packages, including unused
values and functions. This is distinct from source emission: unreferenced fields
and imports are not emitted. Retaining unused owners prevents a conflicting
certificate or consumer-namespace overlap from being hidden by an intermediate
package. Same-authority diamonds pass; different certificates for one Rust crate
fail. Function call heights still measure actual calls, not retained constants.

One shared 100,000-binding budget covers functions and values; existing owner,
qualified-name, total-name, expression and source-byte budgets remain in force.
Constant-variable references are not classified as dynamic loop conditions:
unmodelled compile-time constant control flow fails closed.

## Compiler assembly prerequisite

Follow the Java section of [package/function state](../../rust-source-package-state.md).
Package registration owns the builder, callable/record/import inventories, origin
cache and shared expression budget. Each actual function gets fresh checked body
state; empty body lists emit no method. The 100,000-expression package limit must
survive Reader replacement. Facade and file assembly occur outside a Reader. This
structural prerequisite does not itself admit source constant APIs.

## Single-crate compiler publication

PublicConstants uses a private compiler-evaluated ConstantDeclarationInput and
package State, returning a GeneratedValueId. State retains the exact DefId,
generated value registration, field and evaluated LiteralValue. PublicConstantReads
uses function Reader state to resolve the same DefId and evaluated value into a
typed JavaExprKind::Value reference. Both are required executable consuming-builder
slots; missing/duplicate and wrong capability/context/input/output registrations
fail Rust compilation.

The compiler's closed public inventory registers all fields before lowering any
function body. Mixed facades contain real methods and fields; constants-only
facades have fields and the ordinary private constructor, with no dummy method.
Finite aliases stay in the certified source inventory and share one field identity.
Selected-entry mode deliberately remains value-only and folds public constants;
it does not claim to publish the crate's complete API.

The standalone adapter emits the ordinary Generated.java file. Compiler probes
reconstruct JavaDependencyApi from its certificate and compare values, primitive
types, identities, export bindings and qualified paths before native consumers
compile. No standalone Java JSON sidecar is added by this step. Cross-crate
source joins remain child05B; foreign module constant
reads reject before either creating output or replacing existing output.

## Owned constant bundle schema

Child05A adds Java owner schema 2 when source descriptions include constants;
function-only owners retain schema 1 byte layout. The outer bundle index remains
schema 1. A constant description retains the common declaration identity, module,
location, visibility, documentation and target field path, plus scalar type,
readonly true and exact value. Bool is a JSON boolean; int/long values are signed
decimal JSON strings. No signed value is serialized through floating point.

Projection reconciles each Constant description with the owner's exact opaque
JavaDependencyConstant: original source, qualified path, primitive type, literal
value and public reachability must match. Function and constant public identities
are checked as one complete union; record fields stay distinct from constants.
The existing reservation and encoding sinks walk the same complete metadata.
Whole-bundle checks retain original certificate identities for all owners,
including unused declared members. Foreign source reads/re-exports are still
rejected until the subsequent compiler join/export-closure checkpoints.
