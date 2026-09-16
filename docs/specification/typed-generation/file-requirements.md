# Typed symbol-independent generated-file requirements

- Status: implemented (M35-03A-02F-02B-05C-03)
- First consumer: [C source-package exports](languages/c/rust-constant-reexports.md)

## Unresolved data and plugin contract

TargetFileRequirement<D> contains a target-owned D::ModuleDeclaration and a
RelativeOutputPath. Fields are private; new(module, path), module() and path()
expose descriptive unresolved data, not an import certificate. The module handle
retains language-specific ownership identity; path equality alone is insufficient.

LinkerDialect::file_requirements(&TargetFile<Self>) -> Vec<TargetFileRequirement<Self>>
defaults to empty. Implementations derive requirements from their typed file AST
or frozen package metadata, never rendered text, arbitrary filesystem discovery
or caller-authored resolved directives.

## Shared resolution

Resolve a requirement against exactly one source file in the same TargetAstPackage
with both the requested output path and equal module handle. Reject absent,
ambiguous, mismatched-owner and self targets. Count raw requests (duplicates too)
against a checked 100,000-request package budget. No external package or missing
output is implicitly loaded.

Unify these edges with generated-symbol-reference edges and deduplicate by exact
TargetFileId. Apply the existing source-role rules: runtime may require runtime
only, ordinary sources may not require tests, public API may not require private
implementation files. Named references retain their separate symbol visibility
checks; file requirements never mint a symbol or grant access to private values.
Run cycle checks over the complete graph using an iterative traversal. A dialect's
existing explicit per-cycle policy still applies. An acyclic graph or a forbidden
first cycle uses the fast iterative DFS. If a dialect permits that cycle, inspect
every simple directed cycle, canonicalized by least file ID without reversing
direction. Count each root and examined edge against a separate 100,000-step
selective-policy traversal budget; exhaustion rejects with TargetResourceLimit,
never certifies a partially inspected graph. Both linking and reconstruction use
this same validator. Dense permitted-cycle graphs can therefore be rejected by
the conservative proof-work bound.

Resolve directives only through resolve_file_import(source, destination), yielding
the existing privately constructed ResolvedFileImport. Requirements do not allocate
names, alias imports or manufacture declarations. Independent linked verification
rederives requirements from original unresolved metadata and compares exact edges
and directives; coordinated deletion of both is still an error.

## Language specifications

C17: a GeneratedSource belonging to an explicit CSourcePackage requires that
package's registry-branded public header and canonical output path. Its header
has no reverse requirement. A symbol that already requires the header shares
one edge and include. CGeneratedHeader resolution, include grammar/guard checks
and the structural import renderer remain authoritative. This change alone does
not admit empty or alias-only C packages.

Java21 and the other current plugins retain empty file-only requirements and
unchanged symbol-derived behavior. Their future file-only mappings must implement
this hook with their own typed module identities and satisfy the same shared
proof; no global guessed import strings are added.

## Required evidence

Zero-symbol file graphs, duplicate and mixed symbol/file edges, missing targets,
wrong module owners, self edges, forbidden roles, cycles and deep acyclic chains,
request limits, and coordinated resolved-edge/directive mutations. C public
packages must retain identical native output. Full Bazel release and lint gates
and independent review remain required.
