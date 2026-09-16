# Checked Rust-crate C packages

- Status: normative M35-01D-03 implementation contract
- Parents: [crate visibility](rust-hir-files-and-visibility.md),
  [HIR mappings](rust-hir-mappings.md)
- Compatibility: the existing single-unit experiment remains available

## Shared file-link layer

Reuse TargetFile, TargetFileId, FileItemRoots, LinkedFile and the existing
generated-symbol/file-dependency graph. A generated symbol has one primary
declaration file; referring to or defining it elsewhere does not declare it
again. Cross-file visibility and cycle checks remain authoritative.
[Explicit source-package provenance](rust-constant-reexports.md) also derives a
[typed file requirement](../../file-requirements.md) from the source to its own
public header, independently of whether any owned symbol creates that edge.
Shared resolution authenticates both the registry-branded file and output path;
it deduplicates this requirement with ordinary symbol-derived header edges.

Add a distinct shared resolved-file-import witness alongside ResolvedImport.
It retains the exact destination TargetFileId and a dialect-owned import kind.
Only the shared linker constructs it, from actual checked file-dependency
edges and an executable dialect policy over source/destination module facts.
One dependency destination produces at most one file import. It does not
allocate or alias a symbol: existing generated-symbol bindings remain exact.
Post-link verification reconstructs the complete file-import list, rejecting
missing, extra, duplicate or retargeted imports and changed import kinds.

Dialects whose generated files require no import may explicitly select no
directive; this preserves Java's current same-package behavior. C opts into
generated-header imports. Neither a caller-authored include list nor a generic
default claiming to implement C dependency semantics is permitted.

StructuralImportRenderer spells both linked symbol imports and linked file
imports. It receives resolved witnesses, never sibling-file lookup work,
arbitrary include strings or unresolved target references. File import grouping
and path selection happen before rendering, not in the formatter.

Shared source-file storage is in canonical path order and file groups are in
canonical role order. C grammar traversal independently visits the header
before implementation declarations/definitions. Join inventories by exact
CFileRef identity; never zip independently ordered file/dependency lists.

## C file/import layer

Retain CFileRef, CFileRole, CSourceFile and existing declaration/definition
nodes. Extend the language file grammar to distinguish implementation units
from checked headers with a typed guard. Do not add a parallel C file AST.

C's import kind distinguishes standard headers from generated-header
references. A generated-header witness retains the authenticated CFileRef and
a validated quoted-include path derived from output placement. It cannot name
a source file, refer to another registry or accept directive syntax. Reject
quotes, newlines, backslashes and other unimplemented include-name forms.
The first paired package uses a header and implementation at the same output
directory level. The typed C projection accepts independent registered header
and source basenames: exact CFileRef edges, not filename inference, connect them.
The Rust package frontend (M35-01D-03C) derives deterministic basenames from
crate identity; this naming responsibility does not belong to the C AST layer.

Generated header basenames must be exactly lowercase `polyrust_`, a nonempty
suffix, and `.h`, within the existing portable path and encoded guard budgets.
This reserves a generated-output namespace in the pinned toolchain environment:
neither its standard nor implementation headers occupy that namespace. A finite
standard-header blacklist is insufficient because transitive implementation
headers can also be shadowed by `-I`. Caller-supplied dependencies must not
occupy this namespace; this is not a promise about arbitrary third-party headers.

Header guards are typed identifiers derived deterministically and injectively
from the header's canonical output identity, not a lossy uppercase filename.
The same guard must be used for opening and closing the file wrapper. Guards
must not collide with admitted generated/public identifiers or standard names.
Guard identifiers and include bytes participate in resource admission.

## Whole-package projection and verification

One immutable package projection owns the frozen registry and complete source
inventory. Per-file units refer to that authority and retain exact primary
declarations, used symbols, documentation and file placement. Do not duplicate
the complete declaration set into every file or authenticate each file as an
independent partial registry.

Register public function identities against GeneratedPublicHeader. Their exact
definitions live in the owning GeneratedSource; private function prototypes,
definitions and record layouts remain in the implementation. Attribute
parameters and locals to their defining source body, not to the function's
primary header. Every prototype/definition/owner/linkage obligation is checked
against the whole package. An implementation may satisfy prototype visibility
through its checked header dependency; unrelated headers do not count.

Reconstruct the entire shared graph at package verification and post-link
verification. Reject extra/missing files, declarations, definitions, symbols,
imports, guards and mismatched per-file snapshots. Preserve the one-unit
regression path. Unsupported multi-unit shapes remain diagnostic until all
package resource and rendering obligations exist.

Measure each file and the package as a whole. Source/comment/node budgets are
not reset to permit unlimited aggregate output. Derive the live call graph
across all definitions, including functions whose primary declaration is in a
header. Count shared header syntax conservatively without inventing executable
header frames. Check actual formatted header/source bytes against estimates.

The implementation separates node effects, checked aggregate arithmetic and
typed wrapper/import charges. Guard identifiers participate in the identifier
limit; all three guard occurrences and directive punctuation consume source
bytes. Imports are measured from their resolved kinds. Preprocessor directives
do not invent executable frames. Check the per-file and summed package budgets
before allocating formatted strings, then compare exact formatter lengths with
both estimates before issuing the shared certificate. The one-file frame/node
policy and its native boundary evidence remain unchanged.

## Rust package frontend and API manifest

Package mode is distinct from the legacy selected-score native experiment.
It admits a nonempty set of externally reachable, ordinary nongeneric scalar
functions, plus their reachable same-crate helpers. FunctionSignatures handles
all package function signatures; EntrySignatures remains the legacy test ABI.
All compiler analysis, privacy and declared-input checks still precede output.

Inspect the finite compiler export graph in all namespaces. Every public
non-module binding in package mode must have an implemented API mapping;
currently only ordinary scalar functions qualify. Public structs/constants,
aliases to unsupported declaration kinds, macros, foreign bindings and other
unimplemented API categories diagnose rather than being silently omitted.
Module alias cycles stay finite metadata edges. Multiple public aliases to a
function map to one exact C symbol/prototype/definition, not duplicate bodies
or an infinite list of exported path strings.

A typed manifest records crate identity, admitted compiler export bindings,
exact C function references, file ownership/roles and private/external linkage.
Validate source-to-target and target-to-source membership before serializing
metadata. Private fields/layouts never become public just because a member has
Rust pub visibility. Public functions behind private ancestors can be exported
without publishing those ancestors' private documentation.

Route primary public item docs to header declarations and private item/field
docs to the implementation. Public-reachable module documentation belongs to
the public side; private ancestor documentation stays on the implementation
side. Keep exact module/file owners and deterministic order; definitions must
not duplicate primary declaration docs.

Before paired documentation routing, require the compiler export root to agree
with every owner's crate identity and first module ancestor. Validate the finite
graph once per distinct crate: exactly reachable local module keys, an entry for
the root and every reachable local module, Type-namespace module edges, bounded
bindings/name bytes, and no orphan or foreign keys. Cycles are finite and foreign
module edges remain unexpanded. Reject conflicting owner snapshots. These are
internal consistency checks; metadata alone never proves compiler analysis.
Legacy one-unit documentation does not split visibility and may retain an empty
export inventory, but its root must still agree with the documented ancestry.

The compiler export inventory additionally carries shared root-to-owner ancestry
for each reachable local module, including modules containing only re-exports.
Require exact key equality between this ancestry inventory and the local module
graph before routing; checking only supplied ancestry entries is insufficient.
Package lowering must not omit their doc attributes simply because no generated
function has that module as its lexical owner. Route these through the same typed
module attachment inventory, with exact-owner/root/parent consistency and bounded
ancestry traversal. Do not flatten cyclic alias paths into duplicate comments.

Expose a package-output mode that produces the certified header/source and
validated manifest together. Unsupported input must neither create partial
output nor overwrite existing output. Keep generated examples visible in the
ignored workspace output directory after proof, not only inside Bazel.

The experimental CLI selects package mode with `INPUT OUTPUT --package`, before
any `--input` pairs. It publishes a new flat directory containing the checked
header/source and `api.json`. Existing destinations (including symlinks) are
rejected, even for valid input; automatic replacement is not part of this mode.
The caller exclusively owns the output destination during publication. Stage
all three files in a fresh sibling directory and rename only after complete
writes; failed staging cleans only transaction-owned files. Source errors never
start staging. This is not an adversarial-filesystem/concurrent-writer sandbox.

API metadata has an independent conservative 8 MiB encoded-byte budget, checked
before formatting. Stable compiler identities are serialized as fixed-width hex;
registry authentication brands are never serialized. Function symbols come from
a read-only definition view requiring RenderReadyPackage, not guessed prefixes.

Bazel generation declares the entire directory as one tree artifact. Because
Bazel pre-creates that directory, the action invokes the CLI with a fresh
action-local destination, then transfers the complete three-file result into
the empty Bazel-owned output. The action must not relax the CLI's refusal to
overwrite an existing caller destination. Dependents receive the artifact only
after successful action completion.

## Required evidence

- Shared file-import positive/negative reconstruction and private-constructor
  compile-fail tests; existing Java/default-no-import regressions.
- Hostile header path/guard tests and unchanged single-unit C tests.
- Exact package ownership, primary declarations and file edges; mutations for
  widened visibility, leaked private layouts/docs and retargeted imports.
- Actual-AST package aggregate limits, missing call frames and overflow tests.
- Rust/C parity for all admitted public functions, aliases and private helpers.
- Independently compile implementation and C consumers using only public
  headers; include headers twice and vary supported include order.
- Exact native symbols, manifest bindings, documentation placement and
  three-render determinism; no-create/no-overwrite rejection controls.
- Pinned GCC/Zig, O0/O2, ASan/UBSan, Clippy, rustfmt, buildifier, documentation,
  source-policy and full release gates, followed by fresh independent review.

The paired native oracle compiles the certified header and implementation
unchanged, then separately compiles/links a consumer using only the header
(including it twice). Its exact external function-symbol set must match the
public API. A separate test-only copy retains function addresses for the strict
frame oracle, using the existing direct-call probe mechanism; optimization must
not silently remove required frame reports. Both copies execute with the same
consumer under the controlled 1 MiB stack, at every compiler/optimization and
sanitizer configuration. Address-retention declarations never enter generated
production output.

Separate dependency-crate generation/linking and dynamic-library export policy
remain M35-01D-04. This contract does not silently flatten another crate.
