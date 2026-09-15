# Rust public scalar constants in Java 21

- Status: normative; producer and consumer complete; source integration pending
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
The Java bundle projection/serialization fail closed for Constant descriptions
until their dedicated integration milestone. Existing source kinds stay enabled.

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
