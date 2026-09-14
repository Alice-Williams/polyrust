# C17 module ownership

- Status: normative for M34A-11
- Reference structure: the completed Java backend, not Java grammar or runtime code

## Production modules

| Module | Owns | Must not own |
| --- | --- | --- |
| `ast/` | C types, declarators, expressions, statements, declarations, lexical/type checks | CoreIR decisions or rendered source |
| `capabilities/` | One executable mapping per supported portable capability and its exact plan | Independent duplicate mappings |
| `preflight/` | Dynamic admission from the same registered mapping slots | Rendering or support guessed from target strings |
| `lower/` | Verified CoreIR traversal, symbol requests, sequencing and mapping input assembly | Operation semantics implemented outside their owning mapping |
| `dialect/` | C catalogues, reference identity, includes, namespaces and placement | Portable evaluation |
| `ownership/` | Borrow/owner/allocator/initialization/cleanup proof states | Textual cleanup snippets |
| `runtime/` | Structural scalar, text, bytes, collection and lifecycle implementations | Embedded C/header source |
| `resources/` | Linked target capacities and checked size/layout accounting | Syntax repair or arbitrary portable arity caps |
| `render/` | Certified C grammar spelling, declarator parentheses and formatting | Includes discovery, ownership decisions or compiler processes |
| `tests/` | Focused Rust tests, mutations and compiler harnesses | Production library inputs |

Root files are entry points and explicit re-exports. C does not acquire Java
classes, inheritance, exceptions, packages, JVM limits or JDK catalogue types
through copying. Shared phase machinery is reused; C grammar is independently
modelled with closed enums and private invariant-bearing wrappers.

The C declaration registry is authoritative and frozen only after its complete
declaration/helper inventory is known. The later CDialect/shared unresolved
package owns that immutable payload; shared GeneratedType/Callable/Value/File
registrations are a checked projection, with bidirectional identity bindings,
not a second independently authored source of signatures. No mutable registry
view survives verification. Ordinary Rust equality authenticates references;
address-free canonical inventory projections determine stable ordering and
phase evidence. Ephemeral registry authentication never names an output symbol.

Stage 02A admits generated registrations only. Stage 02B introduces distinct
closed known-object/constant identities and their actual AST type categories;
02D-00 adds the authoritative standard-call signatures/obligations and header
identities required by safety checking. Stage 03 consumes that same catalogue
for shared binding and dependency resolution; it does not duplicate signatures.
The [safety analysis](safety-analysis.md) specifies the internal proof boundaries,
and [known-call contracts](known-call-contracts.md) fixes the initial inventory.
Known references cannot be fabricated by assigning library spellings to
generated keys or by attaching caller-supplied type/effect flags.

## Size and build policy

Aim below 500 lines and split before 1,000. No numbered fragments or wildcard
parent imports. A cohesive test corpus may exceed the soft aim, but production
code and fixture ownership remain separate.

Bazel excludes `src/tests/**/*.rs` from the production library and includes
them explicitly in its Rust test target. Module splitting alone does not create
separate compilation actions. Extract a crate only at an acyclic, stable
dependency boundary with demonstrated cache benefit.

The old `generator.rs`, raw runtime files and fragment adapter are migration
debt. They remain the explicitly legacy route until atomic cutover; new code
cannot call them as a fallback, parse their output or wrap it in a certificate.
No new empty directories or nominal modules count as an implemented layer.

## M35 compiler-source shared projection

The first compiler-source profile uses one registered `.c` compilation unit
(GeneratedSource or TestSource). The file item owns an immutable existing
CSourceFile and CFrozenRegistry, not a replacement C grammar or source string.
Bidirectional typed bindings project every admitted struct, function, member,
parameter and local into shared registrations. Lexical scopes remain owned by
the C registry and are not misrepresented as emitted value declarations.

The shared TypedAstDialect::verify_package hook reconstructs the canonical
projection and compares the entire package: registrations, signatures, origins,
file metadata, groups and payload. Missing and extra entries are equally invalid.
Shared linking and post-link verification repeat this hook. This is validation
of the lowered representation, not a substitute for rustc source analysis.

The first closed grammar admits scalar parameters/results (I32, C Int and Bool),
scalar-field structs, initialized locals, shared object pointers, reads, member
access, comparisons, explicit numeric/qualification conversions, if/else blocks
and value returns. Other categories diagnose; unimplemented catalogue categories
are uninhabited enums. An iterative depth/node guard precedes recursive checks;
it is a verifier protection budget, not a demonstrated target compiler capacity.

Standard dependencies come from the C dependency traversal. Required typed
platform assertions query size/alignment of Bool, Int, I32, Size and void
pointers. Their I32 and Size type references require int32_t/Stdint and
size_t/Stddef bindings even when a function body uses only Bool or Int.
The linker allocates generated names with a poly_ prefix and preserves the
standard typedef spelling. Private collisions receive linker-owned names;
symbol identity never comes from the resulting string.

The closed profile now enforces the measured resource policy and exposes
CStructuralRenderer only through CertifiedSourceFile. The experimental
frontend uses this shared certified route, with no miniature-renderer fallback.
Public headers, crate/API mapping and their separate evidence remain unfinished;
this one-file proof is not the production crate-boundary implementation.

Private structural spelling is split into declarators, expressions, statements,
compilation units and imports. The linker projects names to exact CIdentifier
maps keyed by authenticated C references; spelling does not resolve shared
symbols. Post-link checking rebuilds those maps from linker-owned bindings.
StructuralImportRenderer accepts only resolved imports, and dependency-source
policy permits directive strings only inside its render_imports implementation
(or the existing explicit import-template/legacy-renderer boundaries). Other
methods in that implementation remain prohibited from embedding directives.

The source-policy scanner is a lexical regression guard, not a semantic Rust
verifier or a proof of arbitrary string-building computations. Its bounded
concat handling covers string-literal-only invocations, not char/numeric/mixed
or runtime computations. The new structural import boundary requires the
absolute external trait path `::portable_codegen::StructuralImportRenderer`;
a local same-named trait does not inherit permission. Closed target nodes,
phase certificates and tested trusted renderers establish generation guarantees;
the scanner is supplementary evidence and cannot replace them.
