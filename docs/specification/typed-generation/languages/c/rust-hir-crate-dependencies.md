# Separately checked Rust crates in C

- Status: normative M35-01D-04 implementation contract
- Parents: [crate boundaries](rust-hir-files-and-visibility.md),
  [checked public packages](rust-hir-public-packages.md)
- Scope: explicit generated dependencies with the existing scalar public ABI

## Compiler identity and inputs

Keep the crate, not a source filename, as the compilation/API boundary. Package
generation accepts a validated crate name and explicit stable disambiguator.
Pass both to the pinned compiler; obtain StableCrateId and DefPathHash from
rustc. Never invent a parallel source identity from a path or transient DefId.
The compiler configuration is one typed owner reused by extraction and declared
metadata/native compilation actions. Do not forward arbitrary caller flags.

Default selected-entry and one-crate experiment invocations remain compatible.
Explicit package identities require both name and disambiguator; reject partial,
duplicate and malformed configuration before touching output. The initial name
grammar is ASCII identifier-shaped and the disambiguator is bounded printable
build identity, not an implicit absolute workspace path.

The implemented CLI uses `--package --crate-name NAME --crate-key KEY`, with
`--package` first and the remaining option/value pairs order-independent. Names
are 1..64 bytes, start with an ASCII letter/underscore and continue with ASCII
letters/digits/underscores; the single underscore is rejected. Keys are 1..256
bytes from ASCII letters/digits and `_./:@-`. The name is passed as
`--crate-name=NAME` and the key as `-Cmetadata=KEY`. Input crate attributes still
undergo rustc's checks; passing this lexical boundary is not successful analysis.
The package Bazel rule has paired `crate_name`/`crate_key` attributes. Omitting
both retains the `poly_input` experiment; supplying only one is invalid.

Package symbols use `poly_fn_<crate-id-hex>_<declaration-hash-hex>` after linking.
Both hex components are fixed-width 16-hex-character rustc identity values.
The selected-entry `poly_score` ABI and legacy helper naming remain unchanged.
These remain experimental package APIs, not a promise of ABI compatibility with
the earlier single-crate-only artifact names.

Public-package function names include both stable crate identity and local
declaration hash. Private symbols retain their crate ownership too. Re-exports
retain one defining symbol. Different crates containing identical item spellings
must not collide; ambiguous identities or linker-name collisions diagnose.

Every dependency is a declared Bazel edge carrying its root, additional source
inputs, compiler configuration and compiled Rust metadata. No ambient -L crate
search, host discovery, serialized registry brand or dependency source crawling.
Compiler and generator actions track every source/doc/metadata input so changes
invalidate the appropriate actions with test-result caching still enabled.

The pinned metadata protocol must request `--emit=metadata` for both the
metadata-producing compilation and the source re-analysis. Re-analysis stops
after checked analysis rather than emitting a file. The compiler crate hash
includes output configuration; comparing the old selected-entry invocation
directly with a metadata build would incorrectly reject matching source.

Compiler content identity is also source-filename-sensitive. Dependency actions
must agree on declared logical source names and explicit physical-to-logical
path remapping. Never discard the content hash to work around sandbox paths.
The 04C-01 probe demonstrates stable hashes for equal logical paths under two
different remapped source directories, and unequal hashes for an actual logical
filename change. The production input provider must encode this mapping and
track all mapped source/doc inputs, rather than deriving it from ambient paths.
These hashes are pinned compiler consistency evidence for declared trusted build
inputs, not cryptographic signatures authorizing arbitrary external metadata.

## Dependency authority

An API JSON file is descriptive output, never authority to import executable
code. The first implementation reconstructs each dependency's target certificate
from its declared source inputs in the current driver process. Its successful
compiler analysis must agree with the exact dependency metadata used to analyze
the consumer: compare compiler crate identity/content identity and the fixed
configuration, not only crate spelling or exported signature strings.

Retain separate checked packages. Verifying a dependency again does not permit
copying its bodies, private layouts or documentation into the consumer output.
Emit each crate's implementation exactly once in its own package. Reject cycles
or unsupported dependency graphs before publication; no optimistic recursion
or unbounded dependency reconstruction.

The language-owned dependency API has a private constructor requiring an actual
RenderReadyPackage<CDialect>. It retains exact exported function identities,
registered scalar signatures, resolved linker names, public-header identity,
platform/resource policy and checked call-effect/resource evidence. It cannot
be built by deserializing API JSON or supplying a caller-written purity flag,
signature, frame bound, symbol string or header path.

Public lookup is by compiler crate/declaration identity. It cannot return private
functions or fields. Exact membership, linkage, signature, primary file and
defining package are rechecked from the certificate. Alias edges do not create
additional declarations or change the defining crate.

The first read-only implementation is `CDependencyApi::from_certificate` with
owned RenderReadyPackage input. It retains the immutable owning package by Arc
and issues private-field `CDependencyFunction` witnesses only for exact public
i32/bool signatures. The original CFunctionRef, linked identifier, typed generated
header and implementation owner remain accessible without string reconstruction.
Unknown/private/foreign declaration lookup returns no witness. Cloned function
witnesses retain their certificate even after the API collection is dropped.

At this slice the dependency effect check reuses ScalarCalls on the certified
package's actual definitions. Each public function must have a closed scalar-call
proof. The reported stack bound is the existing conservative whole-package
maximum from complete-package resource measurement, not an exact per-function
frame or a zero-cost external call. Registration is a separate layer below.
Reading the original package (which contains its own private implementation)
does not create a dependency witness for a private function. The API checks
one-crate provenance/export consistency, not independent rustc authentication.

## Typed imported calls

Reuse CFunctionRef, CFunctionType, CCallable and existing call expression nodes.
An explicit certified-import registration binds a dependency function to the
consumer registry and retains the exact original dependency witness. Ordinary
registration still requires a local definition; only authenticated imported
functions may satisfy that obligation with a separate certified package.

Preserve the distinction between owned declarations and imported dependencies
in registry inventories, context checking, shared linking and post-link
reconstruction. Do not create fake local definitions, empty source files or
raw external prototypes to bypass a missing definition. A dependency reference
is not an unrestricted escape from registry branding.

`CRegistry::import_function(CDependencyFunction)` derives the consumer reference
without accepting a new signature, key, name or file. Its contract retains an
opaque exact certificate identity; matching compiler identities or function
registrations do not make independent certificates interchangeable. Imported
functions are exposed through a separate read-only inventory. Their original
foreign public header is not registered as a consumer output file. Parameters,
scopes, statements, local prototypes and bodies require owned functions.
Function addresses, indirect-call contracts, callable members and interface
method/clone/drop roles also remain owned-only in this direct-call slice.
Duplicate imported declaration identities, exact owned declaration-key
conflicts, selected-import/selected-import native-symbol collisions, mixed
certificates for the same crate and consumer/header path collisions diagnose
before registration mutates. All actual allocated consumer-name conflicts and
complete owning-export collisions are linker obligations; registration must not
duplicate the shared linker's prefix/alias allocation.
Header collisions compare actual emitted include names, not only canonical
artifact paths: different directories cannot hide conflicting basenames. Check
dependency/dependency and consumer/dependency collisions in either registration
order. Linking rejects overlap between owned Rust-source crate identities and
imported owners, even with distinct files/declaration IDs and unused imports.
04B-03 admitted projection and linking with a temporary final-certification
barrier. 04B-04 replaces that barrier with mandatory certificate-closure and
transitive-stack reconstruction; ordinary local checks are not bypassed.

The shared language/link layer must carry the exact registered external symbol
and dependency witness. A C imported symbol retains the dependency's linker
spelling; it must never receive an automatic alias that changes its native
linkage. Conflicting spellings diagnose. Dependency headers derive exclusively
from actual imported references and the authenticated public-header witness.
Deduplicate identical includes, forbid private headers, and retain the reserved
header namespace and path budgets. Rendering only spells resolved imports.

Public scalar headers need no dependency implementation or private layout.

A registered dependency reserves the complete public native symbol inventory
of its certified owning header/object, including exports not selected as calls.
Check that inventory against other owning certificates and actual allocated
consumer callable names. Deduplicate owners by exact certificate identity;
reserving an export must not invent an imported call, prototype, or output file.
The existing scalar public-header profile rejects layout declarations before
certification, so this slice does not admit an unchecked additional tag namespace.

The shared C mapping uses opaque `CImportedCallable` and `CDependencyPackage`
values. The callable retains the consumer CFunctionRef and original dependency
witness; the package retains original crate root, public-header identity and
exact certificate identity. Registration-derived catalogues include all imports,
while each file's symbol roots contain only actual imported calls. Owned
declaration inventories never acquire imported functions. C dependency include
records retain the owning package, and the renderer deduplicates typed header
directives without resolving or inventing them.
Calls in a consumer implementation require its own public header and the public
headers of the dependencies actually used. Unused dependencies do not invent
include edges. No dependency source body is a consumer declaration or output.

## Safety and resource composition

The admitted boundary remains safe nongeneric ordinary Rust functions over
i32/bool. No heap ownership, reference arguments/returns, mutable shared state,
callbacks, FFI or arbitrary external library semantics are implied.

Use the existing scalar-call analysis and whole-call-path resource accounting.
A dependency call obtains its effect and stack obligations from the dependency
certificate. It is not an unknown zero-cost leaf. Compose checked bounds with
the actual consumer call graph using checked arithmetic, including transitive
dependency costs; reject absent evidence, cycles and over-budget paths.
Keep all local target verification passes mandatory and unchanged for ordinary
packages. Source checks, target checks and certificate dependencies are distinct
obligations; an exported Rust function alone proves none of the C obligations.

04B-04 validates the complete retained certificate graph iteratively before
resource certification. Exact repeated authorities may form diamonds; distinct
certificates for the same source crate, consumer identity in its dependency
closure, or conflicting complete exported symbols/header names reject.
Unused registered dependencies are included in identity validation but do not
create live call edges or source includes. Prototype verifier budgets are 1024
distinct certificates and 100000 examined import edges; exceeding either
diagnoses rather than recursing or silently dropping evidence.

Reconstruct every certificate's cached whole-package stack bound using its
original typed source and authenticated child costs. Check every child separately
so cached underestimates cannot hide at a transitive leaf. This is not recursive
re-certification: immutable RenderReadyPackage remains the local syntax/safety
authority. Only actual external call targets enter the consumer's temporary
weighted graph, with their already-composed whole-package upper bounds; exported
frame inventories continue to describe owned definitions only.

## Driver and publication

The [declared driver contract](rust-hir-dependency-driver.md) specifies the typed
configuration layer, closed record protocol and following filesystem stages.

Resolve a foreign HIR callee using TypeckResults and rustc DefId, then join its
stable identity and scalar signature to the exact checked dependency API.
Do not read or lower a foreign body as a consumer function. Source privacy errors
still fail rustc first. Valid but unmapped foreign items diagnose explicitly.

Keep the owning package manifest and add explicit dependency/import inventory:
each imported binding records the foreign owner and exact public symbol/header,
separate from local definitions. Reconstruct both directions from the compiler
and certified target graph. Missing, extra, retargeted or private bindings fail.
Only publish after every package and dependency obligation is discharged.

## Required proof

- Identical source/item names in distinct explicit crates yield distinct native
  symbols; repeated processes and relocated source paths preserve identities.
- Invalid/partial/duplicate configuration rejects with no output mutation.
- Private constructors and wrong-registry import use have compile-negative or
  rejection tests; fabricated metadata cannot mint an imported-call witness.
- Dependency signature, source-content, configuration, identity, header and
  effect/resource substitutions are rejected, including coordinated changes.
- Two native Rust crates and two separately generated C packages agree over
  boundary/differential vectors. Compile implementations and consumers separately
  with GCC/Zig O0/O2 and GCC ASan/UBSan, both header orders and the stack policy.
- Exact native defined/undefined symbols match local/imported inventories.
  Private helpers/layouts remain absent from consumers; aliases share one body.
- Full release/frontend/C/Java regression and lint gates, declared-input cache
  invalidation, visible artifacts and a fresh independent review are required.

Implement in the ordered [M35-01D-04 slices](../../../../plan/tasks/M35-01D-04-c-crate-linking.md).
