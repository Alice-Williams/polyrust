# Rust crate boundaries and visibility in C

- Status: mandatory normative M35 design; implementation pending
- Parent: [C HIR lowering](rust-hir-lowering.md)
- Decision: source files within one crate may be flattened; crate boundaries remain
- Existing types: CFileRef, CFileKey, CFileRole, CSourceFile, CLinkage,
  CStorage, CDeclarationKey and typed declaration/member references

## Boundary and provenance

The crate is the generation, dependency and public-API boundary. Rust's
logical modules and physical source files remain provenance and access-control
information, not mandatory separate C translation units. Flattening them
within the same crate is allowed; merging distinct crates is not.

Each source declaration retains its crate, logical module, source-file identity,
source location, declared visibility and effective exported paths. Read compiler
ownership/privacy facts, not guessed paths or a string search through source.
Restricted visibility retains the resolved scope rather than a public/private
Boolean. Extend the existing typed origin/registry metadata.

Inline/out-of-line modules, path overrides and same-basename files must not
collide after flattening. One file participating in different module instances
retains distinct logical owners. Stable names derive from canonical crate and
declaration identity, not original spelling alone, transient DefId/HirId numbers,
absolute workspace paths or hash-map iteration.

Macro-generated declarations belong to their authenticated logical expansion
owner. Retain available definition/call-site provenance; do not move them into
the macro provider's output merely because a span points there. Unsupported
ownership attribution is diagnosed rather than guessed.

## Default C package for one crate

One admitted Rust crate produces one logical C package, normally containing:

- a GeneratedPublicHeader exposing its effective external API;
- a GeneratedSource implementation containing its flattened admitted bodies
  and private concrete layouts;
- a PrivateHeader only where generated implementation/runtime cooperation
  actually requires one.

Public header/source splitting does not split the crate's semantic identity.
Separate source files are not required to survive as separate output files.
Do not create empty translation units for source files with no admitted bodies.
The manifest records the crate and source provenance/selection inventory.

The linker may order/flatten declarations within the crate while preserving
identity, lexical scopes and behavior. It may not merge a dependency crate's
implementation into the consumer or erase the distinction between public
dependency APIs and private internals. Cross-crate references use registered
crate/declaration identities and the dependency's admitted API.

The current experiment has no general dependency-crate lowering. An external
crate use requires an explicit mapped library operation or supported separate
crate generation; unsupported dependencies reject. Do not silently translate
the entire standard library, or treat matching type/method names as mappings.

Helpers and test harnesses retain typed synthesized origins and file roles.
A runtime dependency is not a reason to lose a user declaration's crate owner.
Specializations/adapters must retain both their originating declaration and
their generated placement owner when those differ.

Per-crate implementation splitting may be added for build capacity or caching,
but must keep this API boundary and use private typed cross-file declarations.
Do not promise per-source cache reuse when generation is a single crate action.

## Visibility mapping

A crate boundary does not replace Rust's module privacy. rustc must still
accept every source access. Flattening output cannot make an illegal sibling
access, private field access or restricted re-export valid source.

| Rust access category | Default flattened C policy |
| --- | --- |
| Externally reachable public function | Public prototype/approved export and exact owning crate definition |
| Private item or pub(crate)/pub(super)/pub(in path) item | Internal linkage where possible; absent from external public API |
| pub inside a non-public module with no exported path | Not public merely because its declaration contains pub |
| Public re-export | Preserve the exported alias/path; retain the original definition's crate and declaration identity |
| Private aggregate member/backing layout | Omit from public concrete layout/accessors unless a separate explicit safe API mapping requires exposure |

Use both declared visibility and compiler effective reachability. Public API
inventory is not all declarations whose visibility spelling is pub. Re-export
aliases must not create duplicate implementations or lose their source owner.
Names may be C-compatible mangled spellings, but the typed manifest preserves
the Rust exported paths and their exact symbol mappings.

Represent source public paths as a finite typed binding graph: each reachable
module owns namespace-qualified public names pointing to exact declaration or
module identities. Aliases and cyclic module re-exports are graph edges, not
duplicate definitions or an infinitely expanded list of textual paths. Traverse
each local module once and retain foreign-module edges as crate dependencies.
The graph does not change a declaration's actual private logical ancestry.
The source inventory is shared immutable provenance; the target manifest must
still prove which bindings were admitted and how each maps to checked C symbols.

The initial source inventory uses RustCrateExports shared by Arc through
RustSourceOrigin. RustExportName combines Type/Value/Macro namespace with the
compiler identifier spelling; RustExportTarget distinguishes Module from
Declaration and retains RustDeclarationId in both cases. Identifier strings are
metadata, never unvalidated C identifier or import syntax. Only public-reachable
local modules are expanded. An item re-exported from a private module keeps that
private owner; an external module or declaration keeps its dependency crate ID.

Extraction reads rustc's resolved module children after successful analysis and
uses an iterative visited-module worklist. It admits at most 100,000 scanned
bindings and 16 MiB of copied names, checking before copying. These limits bound
the inventory extraction, not the compiler's prior parsing/analysis memory.
Field access, trait members and supported callable signatures are separate typed
declaration mappings, not extra names invented by this module-binding graph.
Retaining an export does not itself admit that item for C generation.

Use C internal linkage (static for applicable functions/objects) for non-public
items in the flattened implementation. C types/members do not acquire static
linkage: their privacy is provided by keeping definitions out of public headers.
Public signatures may expose an opaque type without revealing backing fields.

If implementation splitting requires package-private cross-file symbols, use
external linkage and a PrivateHeader, not a public export. Generated references
still obey the recorded Rust scopes. Test-only wrappers use TestSource ownership
and never appear in production headers/export inventories.

C does not reproduce Rust module privacy against arbitrary handwritten C.
Source legality is enforced before conversion; generated public/private API
boundaries are enforced by placement and linkage. Private headers are not a
security boundary. Shared-library packaging, if enabled, must derive and verify
an explicit export policy for the pinned linker so non-API symbols are not
dynamic exports. Reject unsupported packaging instead of advertising hiding.

## Linking and documentation

Derive includes and inter-crate dependencies from typed references. Resolve
same-crate callable cycles through prototypes, not invented source strings.
Separate crates retain independently checked public headers; a public header
cannot depend on private implementation headers.

Documentation follows its authenticated owner and visibility even after file
flattening. Keep module-doc blocks in deterministic module order and retain
their association; public API docs go with the exported declaration and private
docs stay in implementation/private output. Comments never widen visibility.

A typed manifest inventory records crate identity, source provenance, generated
file roles, logical modules, public exported paths and private linkage categories.
Compare it with the actual linked declarations in both directions.

## Required proof

- Multiple same-crate files flatten into one implementation without symbol
  collisions, lost declarations or changed behavior; nested scopes remain.
- Two crates retain separate implementation/API inventories and link through
  admitted public dependencies, never consumer access to dependency internals.
- Inline/out-of-line/path-selected modules and duplicate basenames preserve
  compiler identities and deterministic source mappings.
- Private and restricted helpers remain absent from public headers/exports.
  Test pub(crate), pub(super), pub(in path), private parent modules and re-exports.
- Source-illegal private accesses fail Rust analysis. Private fields never
  leak into public concrete layouts.
- Compile C implementations and an external consumer separately; consumer
  includes only public headers. Check exact symbol/API and dependency inventories.
- When shared libraries are supported, deliberately exposed internal symbols
  fail a dynamic-export-table regression.
- Mutation tests catch cross-crate merging, declaration retargeting, widened
  visibility, private-layout leaks, lost aliases and public test symbols.
- Render three times deterministically. Source/doc changes invalidate declared
  generation actions; crate-level flattening need not preserve per-file caching.

## Rationale

Rust defines crates as compilation/linkage units, while visibility is also
module-scoped. This policy preserves both facts without requiring Rust's source
file organization to dictate C translation units.

- [Rust crates and source files](https://doc.rust-lang.org/reference/crates-and-source-files.html).
- [Rust visibility and privacy](https://doc.rust-lang.org/reference/visibility-and-privacy.html).
