# Declared Rust dependency driver

- Status: normative M35-01D-04C contract; checked-source driver and C join implemented, bundle publication in progress
- Parent: [certified crate dependencies](rust-hir-crate-dependencies.md)

## Configuration is not compiler authority

The independently cached `portable_rustc_configuration` library has no compiler
internals or target backend dependencies. Its private-field `InputMapping`,
`CrateDescription` and `CrateGraph` values represent validated build descriptors.
They cannot issue a source proof, dependency function witness or render-ready
package. A caller may construct them with builders or the closed record parser.

Each crate has an explicit validated compiler name/key, one designated root
source mapping, additional source/doc mappings, a metadata artifact path and
alias-to-defining-key dependency edges. Equal crate names with distinct keys
are allowed. Repeated aliases to one target retain separate alias edges but do
not multiply topological dependency counts or owning implementations.

The complete graph requires one declared root, unique keys/metadata paths, only
declared dependency targets, root reachability and acyclicity. Validation and
topological sorting are iterative. Dependencies precede consumers; independently
ready nodes are ordered by stable key, not caller record order. Metadata paths
cannot lexically overlap any declared source/doc input.

## Closed record protocol

The metadata emitter and separate source checker accept these fixed-arity
records; neither interface forwards arbitrary rustc flags.

```text
--root KEY
--crate NAME KEY PHYSICAL_ROOT LOGICAL_ROOT METADATA_PATH
  --input PHYSICAL_PATH LOGICAL_PATH
  --dependency ALIAS DEFINING_KEY
--crate ...
```

`--root` must be first and occur once. Input/dependency records belong to the
preceding crate. Forward dependency references are permitted and checked after
decoding. Missing fields, duplicate aliases/inputs, unknown options, undeclared
nodes and cycles diagnose. There is no `--extern`, `--sysroot`, `-L`, arbitrary
remapping or arbitrary emission-mode escape in this protocol.

Bazel transports the same records through an always-enabled multiline response
file (`@PATH`); each argument is verbatim followed by LF. The emitter accepts
either direct records or exactly one response-file argument, never a mixture.
The file must be regular, UTF-8, at most 32 MiB including separators, and within
the shared graph argument-count bound before allocating individual records.
No trimming, shell quoting, interpolation or recursive response-file expansion
is performed. This preserves declared graph budgets without relying on OS argv
capacity. See [Bazel parameter-file transport](https://bazel.build/versions/7.0.0/rules/lib/builtins/Args).

Before serializing any field, including fields inherited through provider
closures, the Bazel rule rejects ASCII control characters. In particular LF/CR
cannot inject extra records before the Rust field validators see them. This
framing guard supplements, rather than replaces, the closed Rust grammar.

The Bazel dependency attribute maps each alias string to a provider-bearing
target. Multiple aliases may name the same target directly; provider closure
merging deduplicates the defining crate record, not its alias edges.

Names/keys retain the existing closed compiler configuration grammar. Logical
file paths are 1..4096 ASCII bytes containing letters, digits, `_`, `.`, `/` or
`-`; they are relative, have no empty or dot segments, and roots end in `.rs`.
Physical paths are 1..4096 UTF-8 bytes without controls or `=` (the compiler
remapping delimiter). These are descriptors, not an assertion of file existence.
Metadata paths end in `.rmeta`. Within one crate physical and logical input
names are independently unique; canonical alias checking follows below.

Limits are 1024 crates, 100000 alias edges, 4096 inputs per crate, 100000 total
inputs and 8 MiB aggregate physical/logical input and metadata path bytes.
The record parser additionally checks a 32 MiB total argument-byte budget and
an argument-count limit derived from the record and graph limits. Checked
arithmetic rejects overflow. Hitting a limit is a diagnostic, not truncation.

## Required filesystem and compiler stages

Before any compiler invocation, resolve every declared source/doc and dependency
metadata artifact explicitly. Require regular-file identities, reject canonical
aliases that undermine input uniqueness or output/source separation, and build
physical-to-logical mappings exclusively from the declared records. Filesystem
canonicalization does not replace the existing after-analysis check of rustc's
source map and expansion dependency tracking. Undeclared reads still diagnose.

The metadata action and checked-source analysis must use identical fixed
compiler/output configuration and logical source mappings, as measured by
04C-01. Temporary physical paths are not identities. Direct extern arguments
derive from typed alias edges and declared metadata files. If rustc needs a
transitive search directory, it must be privately staged from the exact declared
metadata closure; searching an ambient directory is prohibited.

Metadata actions resolve only their own source/doc files plus the explicit
dependency metadata closure; dependency sources are descriptors for the later
source-rechecking driver, not extra reads by the metadata action. Snapshot each
declared artifact into the private output stage before invoking rustc. Reject
empty/nonregular metadata, canonical or inode aliases between artifacts or the
current source inputs (including source-to-source inode aliases), more than
64 MiB per artifact, more than 256 MiB total,
or more than 8 MiB of resolved source/logical/metadata path bytes. Copying is
bounded and a size change during copying diagnoses. These snapshots are not
authenticated source evidence; that join still requires compiler identities and
checked source hashes. The compiler search directory contains only snapshots;
only direct alias edges get `--extern` entries. Cleanup removes only owned files
and the empty staging directory, never recursively traversing unknown entries.

The declared action experiment additionally found rustc's working directory in
the emitted metadata bytes. Normalize it to `/polyrust/build`, with per-file
mappings under `/polyrust/inputs/CRATE_NAME/LOGICAL_NAME`. Sort all mappings,
including the working directory, by increasing physical-prefix byte length,
stably breaking equal-length ties. More specific textual prefixes must appear
last: both a declared file and the working directory can textually overlap
another input's filename without a path-component boundary. See the official
[source-path remapping contract](https://doc.rust-lang.org/rustc/remap-source-paths.html).
The working-directory remap grants no permission to read undeclared files:
post-analysis read verification still checks their actual filesystem paths.

Reconstruct dependency certificates in graph order in one process, retaining
owned evidence only between compiler contexts. Join loaded compiler identities
and content hashes to separately checked source evidence before admitting any
foreign callable. Compare actual typed signatures and declaration identities to
the exact CDependencyApi witness; names or descriptive JSON are insufficient.

Each per-crate invocation receives the checked subgraph rooted at that crate,
not all siblings from the outer graph. Derive it iteratively from the immutable
declared graph; retain every reachable edge/alias and exact descriptor, then
reuse the graph constructor's validation and deterministic ordering. Unknown
roots diagnose. This prevents a source check from acquiring another branch's
metadata search scope and keeps its compiler configuration identical to the
separately cached metadata action.

## Source and loaded-artifact agreement

The source-checking operation is distinct from metadata emission. It checks all
declared crates dependency-first without publishing generated code. For each
crate, use the same metadata-mode configuration and remapping as its build
action, verify compiler reads, and retain owned `StableCrateId` and `Svh` values.
No transient `CrateNum`, `DefId` or `TyCtxt` escapes its compiler invocation.

Metadata emission and checking must share a fixed pre-initialization setting
that resolves every explicit dependency, including unused ones. The pinned
implementation uses typed `ExternEntry.force`, not an arbitrary option channel or input
feature exemption. Loading via crate-store mutation in `after_expansion` is
invalid because the store is already frozen. Both compiler operations use the
same shared configuration function before initialization.

For every direct declared dependency, require the expected source identity and
content hash and the exact compiler-reported staged metadata artifact path.
The compiler's alias-use table is empty for force-only loads, so merely looking
up an alias or finding the expected identity somewhere among loaded crates is
insufficient. Used aliases must additionally agree with their resolved crate.
Repeated aliases may share one checked owner; distinct declared keys must not
claim one compiler crate identity. Validate the complete loaded declared closure
and reject missing, swapped, stale, body/doc-modified or configuration-mismatched
artifacts, even when exported names and signatures are unchanged.

The source checker first resolves every declared source/doc and non-root metadata
artifact, rechecking aggregate resolved-path and artifact-byte limits. Metadata
cannot share an inode with any source/doc in the graph or another metadata
artifact. The root metadata descriptor is not consumed: checking its source
works whether that unused artifact exists or not and never writes to its path.
Each compiler invocation uses a fresh private scratch stage, with no publication
method. Automatic toolchain dependencies must resolve to exact metadata/library
files in the pinned sysroot inventory; Bazel's individual file symlinks mean a
canonical directory-prefix check is neither sufficient nor appropriate.

The `rust_source_check` Bazel rule consumes the same immutable provider graph
through the shared control-safe response serializer. Its action declares every
source/doc and non-root metadata input in the graph. Its generated text report
is descriptive build evidence, never an importable compiler/target certificate.
The source checker and metadata emitter have independent compilation actions.

The source-checking orchestration exposes a same-analysis callback for C
lowering in 04C-03. Invoke it only after compiler analysis/read validation and
dependency agreement. Retain owned results between invocations, and return the
whole graph only after every node succeeds. No partial target package publication
is permitted. This does not turn source agreement into a C certificate: C
lowering, signature-to-witness binding, verification, linking and resource
certification remain independently mandatory.

The callback receives only a structurally filtered view of previously checked
results from that invocation's exact dependency subgraph, excluding itself and
unrelated siblings. A dependency's compiler input scope and available target
owners must agree; the accumulated outer graph map is not a callback argument.

For the C integration, this view is an invocation-scoped
`CheckedDependencies<'a, 'tcx, T>` with a private constructor. It binds actual
loaded `CrateNum` values to the exact previously checked results only after
artifact agreement; callers cannot provide a crate-number/name mapping. Its
compiler lifetime is invariant and access is immutable. C binding resolves the
foreign DefId's owning CrateNum through this view before looking up a declaration
or comparing a target signature.

Do not publish a partially checked graph. Metadata outputs and final generated
bundles need explicit collision checks and new-output publication semantics that
preserve existing files on failure. C importing and final bundle publication
remain 04C-03; native Rust-versus-C differential closure remains 04D.

## Separate owned and imported API inventories

The compiler bridge retains separate maps of owned definitions and foreign
bindings, keyed by stable declaration identity. A foreign binding contains both
the consumer-branded CFunctionRef and its exact CDependencyFunction witness.
The manifest reconstructs imports from the render-ready package's immutable
projection registry, once per package rather than once per file. It requires
exact equality with the compiler binding map in both directions, including
certificate identity, signature, declaration, public header and consumer scope.
An owning dependency handle cannot substitute for a consumer import handle.
Imported declarations must not belong to the consumer's crate. Owned definitions
continue to be checked against local compiler declarations, files and linkage.

The read-only CImportedFunction view has no public constructor and accepts only
RenderReadyPackage<CDialect>. It exposes registration authority for inspection,
not mutable registry access or permission to construct unchecked calls.

Standalone api.json retains schema version 1 and its existing fields. Attempting
to serialize imports through that format diagnoses instead of silently omitting
them. Bundle member manifests use schema version 2, preserving the owned fields
and adding an imports array, including an empty array for import-free members.
Each import contains id, owner (owning crate root identity), header (certified
include path), symbol (linked owning symbol), return and parameters. Signature
types are the closed i32/bool vocabulary; unsupported types diagnose. Entries
are ordered by stable declaration identity, never compiler crate numbers,
registry addresses, alias spelling or input record order.

Manifest construction and serialization enforce the existing 8 MiB encoded-byte
policy with checked estimates that include imported symbols, paths and signature
parameters. The serialized identities and signatures remain descriptive: no JSON
parser can recover certificate authority. Bundle layout/publication and native
equivalence remain separate obligations of 04C-03-02 and 04D respectively.

## Complete bundle publication

The graph-output operation is --bundle OUTPUT followed by the same closed graph
records (or one response file). It retains a private-constructor checked graph
only after every source invocation, target certificate and manifest succeeds.
Every declared reachable crate is a member, including unused dependencies.

The flat layout has exactly three files per member plus bundle.json:
polyrust_CRATEID.h, polyrust_CRATEID.c and polyrust_CRATEID.api.json, where
CRATEID is the sixteen-digit lowercase stable crate identity. bundle.json uses
schema version 1 with root and members; each member has root and manifest.
Members are ordered by stable root identity. No physical input paths, compiler
crate numbers, alias-dependent names or certificate addresses are serialized.

Before rendering, validate exact member/manifest ownership, every imported
witness against its actual member certificate, unique external symbols and
unique output paths. Require 1..1024 members and exactly 3*N+1 files. Sum the
backend's checked source-byte bounds, manifest estimates and index estimate
with checked arithmetic; reject more than 256 MiB. After structural rendering,
check actual file membership and bytes against the preflight bounds. No source
text rewriting, manually synthesized imports or foreign definition copying is
permitted. The single-crate path retains its independent three-file guard.

All files are rendered before opening a destination or stage. A private sibling
stage owns only its explicitly created files. Write and sync all files before
an atomic no-replace directory rename. Existing files, directories and symlinks,
including racing empty directories, must survive untouched. Cleanup removes
only owned files and the empty stage, never recursively traverses unknown data.
This promises atomic visibility, not power-loss durability of directory entries.

The pinned Linux launcher supplies a first-party no-replace helper as an explicit
Bazel runtime dependency. The CLI invokes that exact path without a shell or
PATH lookup. Source graph records cannot select the helper. The helper uses
renameat2 with RENAME_NOREPLACE; failure, including unsupported filesystem/kernel
support, is a diagnostic with no overwrite-capable fallback. Rust remains
unsafe-free and no third-party publication dependency is introduced.
