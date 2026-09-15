# Rust public scalar constants in Java 21

- Status: normative design; implementation pending M35-03A-02F-02B-03
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
