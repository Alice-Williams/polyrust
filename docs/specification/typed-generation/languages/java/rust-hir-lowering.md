# Java lowering from compiler-checked Rust HIR

- Status: normative design; single-crate lowering, target dependency bindings and compiler graph integration implemented; production bundle publication in progress
- Target: org.polyrust.java, existing pinned Java 21 toolchain
- Plan: [M35-01E](../../../../plan/tasks/M35-01E-java-rustc-retrofit.md)

## Boundary and authority

The compiler adapter accepts actual Rust source using the pinned configuration
and successful-analysis boundary established by M35-01. It passes compiler
HIR, resolved types, adjustments, identity and documentation to Java-owned
typed mappings. It does not parse a dump, translate generated C, or reconstruct
a generic Rust AST.

The Java backend remains independent of rustc internals. Lowering creates the
existing Java AST through its checked constructors and registered references.
The established Java verifier, linker, post-link/resource checks and
RenderReadyPackage<JavaDialect> gate remain mandatory. No caller-supplied
source-provenance record or Boolean can stand in for compiler analysis.

Rust source is dynamic input. Valid Rust outside the implemented capability
subset must produce a diagnostic before artifact publication. This does not
weaken or redefine the existing typed-builder SupportsAll contract.

## Initial semantic mapping

The first shared source contract is the C no-heap proof subset, not arbitrary
Rust. Entry arity restrictions belong to the experiment harness; the Java AST
continues to support its existing parameter lists.

| Rust source | Java representation and obligation |
| --- | --- |
| i32 | Java int; exact signed 32-bit values |
| i64 | Java long; exact signed 64-bit values and two-slot JVM parameter/local accounting |
| bool | Java boolean |
| Nonempty no-Drop scalar-field struct | Nominal immutable implementation type, constructor and final fields; retain declaration/member identity |
| Initialized immutable binding | Typed local with exact initialization and lexical scope |
| Move/copy of admitted values | Assignment of the selected representation; do not invent cloning or observable identity |
| Shared reference | A Java-owned typed representation plan for immutable access; no null introduction or mutable aliasing |
| Built-in field access/dereference | Resolve the actual compiler field/adjustment through that representation plan |
| Scalar comparison | Java primitive comparison with Boolean result; no reference-identity equality substitution |
| Built-in bool lazy and/or | Initialized private bool temporary and typed If/Assign; right prelude remains branch-local |
| Built-in i32/i64 complement/and/or/xor | Exact int/long unary/binary nodes and precedence enums; eager operands materialized left to right |
| Tail block and if/else | Structured Java scopes, branches and returns |
| Resolved documentation attributes | Escaped documentation attached to the corresponding Java declaration |

Java object sharing is permitted only where no admitted source operation can
observe a difference from Rust value semantics. The initial public boundary
takes/returns primitives, and record representations are immutable and private.
Do not claim a general mapping for identity, mutation, interior mutability,
pointer comparison, borrowed returns, custom Drop or heap cleanup.

Represent shared references explicitly in the lowering type/representation
plan, even if the resulting immutable Java expression is the same as its
referent. Borrow erasure is a proved mapping for a closed use set, not an
untyped expression rewrite. Adding mutable borrows or public reference-bearing
APIs requires a new capability contract and tests.

## Capabilities and dependencies

Every implemented Rust-source capability owns a typed input category and an
executable mapping that produces existing Java AST categories. Use the existing
builder registration discipline. A support flag without the actual mapping,
a wildcard support implementation, or an erased all-purpose input enum is not
acceptable.

Keep source semantics outside formatting. Imports, constructors, standard
library references, helper injection and namespace resolution derive from
typed registrations. The renderer performs structural Java spelling only.

## Java-owned source representation

The adapter's representation enum distinguishes i32, i64, bool, immutable nominal
records and shared-reference referents. It is a target representation plan, not
a reconstructed generic Rust AST. A Java value pairs that plan with an existing
JavaExpr whose JavaType agrees with the erased representation. A place is a
separate typed wrapper admitted only from resolved bindings, authenticated fields
and built-in shared dereference. Erasure must not turn an arbitrary value into a
place or allow reference equality, mutation, nulls or escaping reference APIs.

The [i64 extension](../../rust-i64-values.md) uses primitive long throughout,
including imported method signatures and immutable scalar-field records. Shared
checked literal input distinguishes I32, I64 and Bool values; Java lowering
maps that enum without parsing source token text. Native long literals retain
the exact signed minimum, and source-size/classfile checks include their real
representation. Other integer widths, arithmetic, casts and mutable integer
locals are not enabled by this representation extension.

The [integer bitwise extension](../../rust-integer-bitwise.md) registers
IntegerBitwise with exact int/long unary/binary AST nodes and precedence enums.
The separate [eager Boolean extension](../../rust-eager-booleans.md) registers
EagerBooleans with primitive Boolean bitwise nodes and their exact precedence.
Both operands are materialized left to right, without lazy branch substitution.
The dependency gate and source reservation traverse these operations explicitly.
No support class or runtime helper is introduced.

Literal, scalar-constant, comparison, Boolean-negation, short-circuit Boolean, integer-bitwise, eager-Boolean, borrow, call and record-initializer mappings produce planned
Java values. ObjectTypes produces the representation plan; ResolvedPlaces produces
a planned place; LexicalControl produces JavaBlock. EntrySignatures and
FunctionSignatures produce JavaMethodSignature from compiler signatures. Every
builder slot constrains both its compiler input capability and exact Java-owned
context/output. Shared compiler contracts contain none of these Java types.

Calls and record-field expressions are materialized in source order before any
argument rearrangement. Boolean ordering converts each Boolean operand once via
a typed conditional expression to int 0 or 1; equality requires no conversion.
Compiler-local identities map to fresh target names. A terminal nested Rust block
may flatten into Java's terminal statement sequence when names remain distinct,
source binding scope is retained by the adapter and no computation is reordered.

[Boolean negation](../../rust-boolean-negation.md) has its own required executable
slot and checked compiler input. It maps only built-in bool Not to a typed Java
unary node, evaluating its operand once. Dependency-body admission and source
byte reservation traverse that node without admitting other unary operations.
No runtime helper or import is introduced.

[Short-circuit Boolean expressions](../../rust-short-circuit-booleans.md) have
their own checked input, typed And/Or enum and executable slot. Evaluate the
left value once, initialize a private mutable bool local and place the complete
right prelude inside its conditional block. This is implementation-local
mutation, not support for source assignment or mutable references. Dependency
body admission tracks initialized mutable bool locals per lexical scope/method;
it does not admit writes to parameters, fields, final locals or other types.

The single-crate source adapter initially admits at most 4,096 reachable functions,
100,000 expression visits in call discovery, and depth 128 for discovery and
lowering recursion. Lowering also has a 100,000-step budget and at most 100,000
locals per function. Recursive call graphs reject; acyclic call height is at most
128. These are explicit admission policies, not intrinsic Java/Rust compiler limits.
Existing Java target resource certification still runs after source admission.
The shared C/Java alias-use admission scan is also bounded, including unused
item surfaces: at most 100,000 charged visitor operations and 128 nested guarded
categories. Compiler ControlFlow breaks propagate through all enclosing walks
and the outer item iteration; exhaustion diagnoses before target publication.
Generated scalar calls use the existing pure/non-null/unchecked-exception-free
signature contract only because every reachable body must pass the closed
immutable-computation mapping; purity is not inferred merely from a Rust fn type.

The initial single-unit CLI requires the output leaf filename Generated.java,
matching its public facade. It rejects other filenames before invoking rustc or
opening output, and does not silently rename certified compilation units. Tests
compile the published file in place. Atomic directory bundle publication remains
the separate multi-crate integration step.

## Crates, visibility and documents

The Java package vocabulary retains the legacy Generated namespace and adds
RustCrate(stable_crate_id), spelled org.polyrust.generated.r plus sixteen
lower-case hexadecimal digits. Each nonempty certified target package uses one
Java namespace. Empty certificates render no files or namespace. A mixed-namespace target package rejects; separate Rust crate
certificates join only through typed dependency APIs. This keeps package-private
access assumptions explicit rather than silently treating different namespaces
as one. The existing Runtime identity remains in the legacy namespace and
cannot be copied/relocated into a source crate under any file role. Runtime is
a globally reserved top-level target name; source names require mapping.

Package IDs are bounded naming data, not source-analysis authority. Actual
RustSource declaration/member ownership is authenticated separately. Resource
checks use each actual package prefix, including synthesized JVM descriptors.
RustCrate output carries only a provenance-neutral generated header: its
namespace alone does not prove CoreIR or compiler-source provenance.

Registered RustSource types, callables, interface methods and static values
must carry declaration origins, not body-local node origins. Their declaration,
module, export root and any restricted visibility owner must have the containing
RustCrate ID. A source declaration identity may occur in only one of those
registries. The ordinary package verifier enforces these coherence rules both
before linking and during post-link certification. Legacy and synthesized
target declarations retain their separate origin categories. This registration
check is not a substitute for compiler analysis, export-graph validation,
member authentication or documentation ownership checks.

Immutable source record components use a separate RustSource origin variant
containing the owning Rust declaration and shared field-source metadata.
Their field references carry both the registered Java nominal owner and Rust
field declaration ID, plus typed name and result type. Verification joins those
identities against the actual component declaration and its registered owner;
same spelling alone is insufficient. Source fields cannot impersonate Core or
runtime record members. Existing private-nest access, final assignment,
constructor initialization and resource rules apply unchanged. Explicit canonical
constructors are the initial source adapter's construction representation.

One Rust crate becomes one separately identifiable generated package/API unit.
Logical modules can be flattened within that unit. The package plan records
their provenance, source files and actual effective exports, including
re-exports; it never merges unrelated crates or promotes a private selected
function to the public API.

Use private nested implementation declarations or package-private declarations
where appropriate. Public facade declarations correspond to actual Rust
exports. If Java cannot express a source restriction directly, rustc enforces
source access, while the generated package exposes the smallest practical API.
Do not claim identical protection against arbitrary handwritten target code.

Reuse the shared RustDeclarationId/RustSourceOrigin metadata vocabulary, not
target symbol spellings as identities. Shared type/callable/value registrations
carry `GeneratedOrigin::RustSource`; do not invent CoreIR IDs or relabel source
declarations as runtime synthesis. Metadata is not a source-validity or
target-rendering certificate. Stable naming cannot depend on arena addresses,
hash-map iteration order or absolute checkout paths.

Retain resolved doc attributes; ordinary comments are not promised. Escape Java
documentation terminators and Unicode-escape hazards structurally. Keep the
original text in metadata even when displayed documentation requires escaping.

Source documentation coherence is target-independent and checked in the shared
source metadata layer. A privately constructed borrowed inventory retains unique
declaration owners, canonical module payloads and a coherent finite export graph.
Java must require that check before linking and again during post-link validation.
Metadata text (including diagnostic paths and export spellings), attribute counts,
declaration counts and ancestry/export traversal have finite checked budgets.
Shared allocations are scanned once; independently allocated copies must still
be bounded and compared for equality. A successful metadata check authenticates
neither rustc analysis nor target syntax.

The initial source facade is a synthesized PackageEntryPoint named Generated;
it has an explicit private zero-argument constructor so Java cannot synthesize
a public construction API unrelated to any Rust export.
it is not a Rust nominal declaration. Actual Rust types, functions and fields
retain RustSource origins. Crate/module documentation has module ownership and
must not require inventing a source declaration for that facade. Attachment
routing is a later checked projection, separate from metadata coherence.

Resolved Java file items retain typed documentation attachments alongside their
resolved names. Declaration attachments join actual registered symbols and
source-field identities; re-exports and interface implementations do not copy
primary documentation. Attribute order is preserved within each owner. The one
emitted synthesized PackageEntryPoint is the presentation owner for module docs:
root attributes describe the facade, while other modules remain explanatory
block comments, ordered by depth then stable identity. Nonempty module documents
without that presentation owner reject rather than disappear. This does not
promote private Rust modules into public Java APIs.

The shared linker's exact post-link file-item rederivation includes documentation.
The renderer receives encoded JavaDocComment payloads with fixed structural
delimiters/prefixes. Certification bounds attachment count and complete normalized
comment presentation size, including indentation and line prefixes.
The initial policy is 100,000 attached owners and 64 MiB of comment presentation
per package; exceeding it is a target-resource diagnostic, not an approximation.
Empty-doc
legacy packages retain their historical output. These projection/rendering rules
must be implemented and proved in M35-01E-02D-03 before the source adapter uses them.

## Layer implementation contract

The source-owned alias admission visitor is a separate shared module,
source_admission, not a capability input contract or a concrete emitter.

1. Compiler boundary: the existing pinned full-analysis/configuration driver
   owns successful Rust analysis and exact dependency identity. Shared
   source_capabilities contain only compiler inputs and executable mapping
   traits, one capability per file; no Java or C representation belongs there.
   The shared source_origin reader owns compiler identity, remapped locations,
   visibility, resolved attributes, cached module ancestry and export graphs.
   It returns the existing RustSourceOrigin metadata, not a new AST or proof of
   analysis. Concrete target keys and registrations remain backend-owned. All
   extraction budgets and declared doc/source input checks remain mandatory.
2. Java adapter: java_lower owns representation choices and a consuming builder
   with exact context/output bounds for each of the fifteen admitted input
   categories. It emits existing Java types, not a new generic AST.
3. Java target model: extend the existing JavaPackage with a typed Rust crate
   identity. Its package spelling and path derive from that identity. Preserve
   the legacy Generated/Runtime model as a separate variant. Rust source
   origins and exact nominal/member ownership enter the registered AST.
4. Certification: existing structural/context, binding, visibility, link,
   post-link and resource checks remain mandatory. Rust-source immutable
   records have their own member identities; they cannot impersonate CoreIR
   fields or borrow CoreIR conformance witnesses.
5. Dependencies: Java-owned immutable certified API handles authenticate foreign
   calls and supply typed qualified names/imports. Descriptive JSON is never
   a proof input. The complete checked crate graph is certified before output.
6. Rendering/publication: the existing structural Java renderer consumes only
   RenderReadyPackage. Docs are escaped structured data with resource accounting.
   Bounded complete bundles use the established atomic no-replace publication
   boundary. No target string fallback or handwritten import list is introduced.

The ordered implementation and per-layer proof obligations are in
[M35-01E's child tasks](../../../../plan/tasks/M35-01E-java-rustc-retrofit.md).
Until their individual evidence is recorded, this section is specification,
not a statement that the Java Rust-source path is implemented.

## Certified dependency implementation contract

JavaDependencyApi is constructed only from RenderReadyPackage<JavaDialect> and
retains that exact immutable certificate. Its exported JavaDependencyFunction
handles carry exact owner authority, Rust declaration identity, resolved facade
member identity and scalar signature. Equality distinguishes independently
certified packages even when source IDs coincide; authority addresses never
determine output names or ordering. The API inventory reconciles actual public
declarations with the entire resolved source export graph. It rejects omitted
public bindings, private exports, non-scalar signatures and bodies outside the
admitted closed local-only computation subset. Metadata alone cannot certify purity or
substitute for rustc source/metadata agreement.

Resolved Java file items retain a Java-owned source-declaration inventory derived
from the original typed registrations during linking. Exact post-link item
rederivation includes this inventory, just as it includes documentation and names.
It provides certificate-backed source/export and signature facts to API extraction
without exposing or recovering the shared LinkedTargetPackage's unresolved AST.
It is a typed metadata projection, not another AST or a stand-alone certificate.
Its field, declaration variants and readers are crate-private to backend-java;
external consumers use the certificate-derived dependency API, not the original
registration table. Compile-negative tests enforce this encapsulation.

The projection retains at most 100,000 declarations, 1,000,000 signature
parameters and 64 MiB of declaration-name bytes per certified package. Shared
source provenance remains Arc-backed and subject to existing documentation
coherence/bounds. Closed owner admission separately permits at most 4,096 ordinary
functions, 100,000 body/block/statement/expression visits across the package and
128 levels of body nesting or acyclic call height. These are explicit admission
limits, not a claim that every syntactically valid Java program is a dependency.
Record constructors must consist solely of exact canonical field assignments;
method bodies admit initialized final locals, initialized mutable bool locals
and writes to those locals, returns, total conditionals, scalar comparisons,
built-in Boolean Not and eager and/or/xor, exact int/long complement/and/or/xor, immutable record
construction/reads and closed local calls.
Other mutation, general arithmetic, external/runtime calls and recursion do not acquire
this proof merely by setting a pure signature flag.

Constructor blocks (including the empty facade constructor), canonical assignment
statements and their expressions consume the same package-wide visit allowance
as ordinary methods. Export-graph reconciliation compares each distinct Arc
allocation at most once across both declaration and field inventories; repeated
shared fields cannot multiply a large graph scan. Allocation identity serves only
as temporary comparison memoization, never generated naming or output ordering.

A Java-owned binding scope imports these handles and freezes consumer identity
and registrations into the original target projection. Dependency call AST nodes
contain opaque imported references. Verification authenticates the consumer
scope, exact owner and signature; linking derives the dependency catalogue from
that original projection and post-link checks repeat the derivation. A foreign
arena index or matching name is never a consumer binding.

The shared linker distinguishes fixed-import and qualified dependency spelling
with a typed enum. C retains its existing fixed-import behavior. Java selects a
qualified static-call name composed of its certified RustCrate namespace, facade
identifier and method identifier. Two crates may therefore retain identical
member names without colliding. Fully qualified Java calls require no import
directive; emitting unused static imports or fabricated aliases is forbidden.
Other Java imports continue to derive solely from actual typed uses. New binding,
name and inventory data participates in target resource accounting.

JavaDependencyScope is a consuming builder: import takes a certified function and
returns the updated builder plus an opaque JavaImportedCallable; finish consumes
the builder and returns JavaDependencyBindings. The builder is not Clone. Frozen
bindings are immutable and share exact identity when cloned; separately created
scopes remain distinct even with identical functions. Empty legacy scopes have
no identity. Each Type file item retains its bindings in the original AST.
Catalogue reconciliation checks each item's uses against those bindings, rejects
multiple nonempty frozen scopes, conflicting proofs for the same source crate,
and overlap with consumer source/namespace identities. No writable signature is
stored in a dependency call node.

Per-call type checks read only the opaque owner's certified signature. Frozen
consumer membership is checked once per file over its collected typed symbols,
then independently during original-package catalogue derivation. Signature and
exception checks must not rescan the complete package for each call.

Dependency registration is bounded to 100,000 functions, 1,024 owner proofs and
64 MiB of qualified-name bytes per consumer package. The transitive used-owner
closure of every registration also admits at most 1,024 distinct owner proofs;
it rejects conflicting certificates or consumer identity overlap at any depth,
not just immediate imports. Exact shared leaves in a diamond are visited once.
Each qualified dependency
path is also bounded to 65,535 bytes. Checks include unused registrations.
The certificate-derived API reports exactly the directly used owners, in source
crate identity order, rather than all registered owners. These are generated
package dependencies, not fabricated Maven package-manager entries.

Imported-call admission carries the owner's checked call height into the consumer
calculation; a cross-crate edge cannot reset the 128-level acyclic call-path limit.
The Java scope/call implementation and independent native proof are sequenced in
[E04B's checkpoints](../../../../plan/tasks/M35-01E-04B-java-imported-callables.md).

The compiler graph adapter reuses the existing dependency-first source_check
driver. Each foreign DefId joins the exact loaded crate identity and certified
owner API before its compiler signature is compared and the imported call is
registered. Local bodies are emitted only by their owning crate. Any failed
member discards the complete graph result; serialized manifests are descriptive.

Call discovery returns separate typed inventories of local bodies and foreign
compiler declarations. It visits HIR bodies only for local definitions. Foreign
registrations use the same FunctionSignatures capability as local functions,
compare that complete Java signature and stable Rust declaration identity with
the exact certified owner function, then freeze consumer-owned imported handles.
The DirectCalls mapping selects a local Generated reference or an opaque
Dependency reference; both retain source-order argument materialization.

The separately built java_graph_adapter reuses the source_check module and its
invocation-scoped CheckedDependencies view. Its --check-crates mode performs
full analysis, metadata agreement and complete graph certification, reports only
complete success, and writes no target artifacts. The single-crate adapter still
rejects foreign calls without this authenticated lookup. Test-only fault-injection
adapters are separate Bazel compiler actions, not runtime production options.

The completed compiler graph proof is recorded in
[focused C01/C02/C03 checkpoints](../../../../plan/tasks/M35-01E-04C-java-compiler-graph.md).
Test-only certificate rendering does not establish the production atomic bundle
publication contract below.

Bundle publication admits 1..1024 owners, one source and one owner manifest per
owner plus one bundle manifest (2N+1 payload files), and at most 256 MiB total
bytes. Canonical Java paths/basenames are preserved. The existing bounded staging
and atomic no-replace directory publisher runs only after every owner certifies
and renders; existing or concurrently created destinations are never overwritten.
The implementation checkpoints and proofs are
[M35-01E-04A through M35-01E-04D](../../../../plan/tasks/M35-01E-04-java-crate-bundles.md).
Canonical source trees have at most N+6 directories below the transaction root,
with directory depth seven. The shared publisher has typed flat/tree policies,
validates raw relative ASCII path components without normalization, rejects
duplicate files and file/directory prefix conflicts before staging, and bounds
payload files, unique directories, path lengths, depth and bytes. Caller limits
cannot exceed its hard ceilings (3,073 files, 4,096 directories, depth 32,
4,096 path bytes and 256 MiB). Java's tighter inventory bounds still apply.
Staging creates directories/files exclusively with modes 0700/0600 and cleans
only exact transaction-created paths, children first. The destination is never
traversed for cleanup. The parent/helper are trusted local infrastructure;
crash durability and hostile same-user staging mutation are outside this contract.
Dependency bindings, compiler graph integration and bundle inventory/reservation
are complete. The production graph adapter now accepts `--bundle DESTINATION`
followed by the same declared graph records (or bounded response file) as check
mode. It reports success only after the entire tree is atomically published.
`rust_java_bundle` consumes `RustSourceCrateInfo` and emits a declared Bazel tree;
the pinned compiler and native publisher are declared tools, with no dependency
discovery or host compiler fallback. Production atomic/native proof is complete
in D03; E05 completed fixture, isolated commit-tree and publication closure.
Check mode still renders
and publishes nothing.

The closed Java owner API exposes source_byte_bound before text generation.
Its bounded AST walk accounts for each emitted name occurrence, including the
full typed qualified path of each dependency call. A declaration/parameter/
component/statement/expression visit reserves 256 fixed syntax bytes plus
16 times its structural depth for indentation, with variable spelling bytes
charged separately. The facade/package header is charged as another node.
Already escaped documentation attachments contribute their full presentation
length exactly once, including line prefixes and delimiters. Every addition is
checked against 256 MiB; unknown measurement shapes reject rather than using
fallback estimates. The walk accepts only immutable closed owner certificates
and caps measurement depth at 256 (above the admitted body-depth limit plus
facade/constructor nesting). This is a final-output byte bound, not a claim about
temporary renderer allocation or JVM bytecode. Whole-bundle reservation adds
manifest/index bounds and compares final counts, paths and bytes before staging.

The versioned metadata and canonical file contract is specified in
[Java Rust-source bundle manifests](rust-hir-bundles.md).

## Migration proof

The same compiler-negative, unsupported-input, scope, field-order and borrow
fixtures must exercise C and Java admission. Add direct Java AST assertions,
native Java 21 execution over the shared boundary corpus, separate facade
consumers, deterministic artifacts and hostile-document/name tests.

Historical Java generated examples and builder tests stay enabled until their
replacement coverage exists. A frontend change does not authorize bypassing
the existing Java certificate or discarding previous correctness regressions.
The full C/Java migration gate must pass locally before any push.

## Compiler-evaluated constant reads

[M35-03A-02F-01](../../../../plan/tasks/M35-03A-02F-01-constant-reads.md)
adds ScalarConstants as an executable slot under the shared
[constant input contract](../../rust-scalar-constants.md). Map checked values
to primitive boolean/int/long literal nodes and matching TypePlans; do not
introduce Runtime, boxing, source fragments or synthetic static fields.
Keep exact signed minima and wide integer spelling. Constant evaluation stays
in rustc, not the renderer. Public constant exports still require explicit
API/dependency support in 02F-02B. Block-local declarations are extended below.

## Block-local constant declarations

[M35-03A-02F-02A](../../../../plan/tasks/M35-03A-02F-02A-local-constants.md)
adds the LocalConstants executable slot with private compiler-derived input,
Reader context and unit output. Follow the shared
[local constant contract](../../rust-local-constants.md): evaluate/validate every
admitted declaration, including unused ones, then erase it without registering
a Java local, field, static initializer, helper or import. ScalarConstants
retains compiler identity and exact primitive literal types at each read.
Forward references and nested shadowing use rustc resolution, not target names.
Unknown local item kinds and unsupported constant types reject atomically.
Public exports, borrows and generic constants remain outside this extension.
