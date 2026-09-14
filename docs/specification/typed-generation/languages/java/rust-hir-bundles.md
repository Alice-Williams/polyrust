# Java Rust-source bundle manifests

- Status: implemented and independently reviewed through M35-01E-04D-03
- Parent: [Java Rust-HIR lowering](rust-hir-lowering.md)
- Plan: [typed manifest projection](../../../../plan/tasks/M35-01E-04D-02B-java-manifest-projection.md)

## Authority boundary

Input is the complete compiler-authenticated CheckedGraph. Each member retains
its exact immutable JavaDependencyApi. Metadata is a read-only description;
neither JSON nor a descriptive object can reconstruct an owner, callable or
render-ready certificate. Private declaration descriptions grant no import API.

The backend supplies typed read-only source descriptions derived from its
certificate-retained registrations, resolved declaration paths and record-field
origins. The compiler adapter does not parse generated Java, guess source IDs
from Java names, or expose the backend's original unchecked registration table.
Every public source function must correspond to the exact public API function;
private functions must have no public dependency handle.

The independently cached `portable_java_bundle` utility borrows `Owner { key,
api }` values and validates exact target certificates; it does not authenticate
Rust source or declared dependency membership. Only the compiler adapter's
private `CheckedGraph` establishes that input membership. Supplying arbitrary
target-certified owners to the utility does not constitute compiler evidence.
In particular, disconnected declared-but-unused owners are intentionally retained.

## Canonical files and ordering

For N owners, output contains exactly 2N+1 UTF-8 payload files:

- Each owner's existing certified
  `src/main/java/org/polyrust/generated/r<crate-id>/Generated.java`.
- One `polyrust_<crate-id>.api.json` per owner.
- One `bundle.json` index.

IDs use lowercase, zero-padded hexadecimal: crate IDs have 16 digits;
declaration IDs use `<crate-id>:<definition-path-hash>`, with 16 digits per part.
Owner and declaration lists sort by typed stable IDs, never pointer addresses or
physical input locations. Binding lists sort by namespace and source name.
Documentation arrays preserve compiler attribute order and exact text.
JSON is deterministic, has fixed field order, no insignificant indentation and
one terminal LF. All variable string values use a dedicated JSON encoder.

## Index schema, version 1

The index object has `schema_version: 1`, `target: "org.polyrust.java"`, `root`
(the root owner's declaration ID), and `members` (ordered owner entries).
Each entry has `root`, `source` (canonical Java path), and `manifest` (owner JSON
path). Every index entry has exactly one owner manifest and one source file.
Unused declared dependencies remain graph members and are published once.

## Owner schema, version 1

An owner object contains `schema_version`, `root`, `defining_key`, `source`,
`modules`, `declarations` and `dependencies`.

- `defining_key` is descriptive compiler-graph metadata, never lookup authority.
- `modules` is a flat ID-ordered inventory combining export modules with retained
  source ancestry. Each module has `id`, nullable `parent`, `location`,
  `documentation`, and `bindings`. Cyclic module aliases are represented as
  binding edges, not recursively expanded trees.
- A binding has `namespace` (`type` or `value`), `name`, and a typed target:
  `module` or `declaration`, with its exact stable ID. Alias spellings never
  create additional generated methods or copied owner bodies.
  The target object is `{ "kind": "module" | "declaration", "id": ID }`.
- `declarations` contains the retained source functions, private records and
  their scalar fields. Each has `id`, `kind`, `module`, `location`,
  `visibility`, `externally_reachable`, `documentation` and `target`.
  Functions additionally expose ordered parameter/result scalar kinds (`i32`
  or `bool`); fields identify their owning record and scalar kind. Java target
  spellings derive only from typed resolved paths/component identifiers.
- `visibility` distinguishes `public` from `restricted_to` with its module ID;
  it is separate from effective external reachability.
  Its object is `{ "kind": "public" }` or
  `{ "kind": "restricted_to", "module": ID }`.
- A `location` has compiler-remapped `file`, `line`, and `column`. These are
  diagnostic metadata, never output paths. Relocation must preserve their bytes.
- `dependencies` lists only directly used owner roots in stable order. Every
  imported callable is reconciled with the exact owning graph member's function
  witness, not merely a matching owner/declaration ID.

The synthetic Java facade is described by the owner's canonical source path;
it is not invented as a source-language declaration.

Declaration target objects are `{ "kind": "declaration", "path": PATH }`
or `{ "kind": "field", "owner": PATH, "member": NAME }`. A PATH contains
`package`, ordered `owners`, and `member`, each from typed resolved Java names.
Function `parameters` is the ordered scalar-kind array and `result` its scalar
kind. A field additionally has its source record ID in `owner` and kind in
`scalar`. No Java code is parsed to produce these objects.

## Reservation and publication

Admit 1..1024 owners and at most 256 MiB aggregate final UTF-8 bytes. Canonical
directory prefixes number at most N+6 and have depth seven. Count the index and
owner manifests as payload files, separately from those directory prefixes.

The fixed source path has six shared directory prefixes (`src` through
`generated`) plus one `r<crate-id>` prefix per distinct owner: exactly N+6
directories, with maximum depth seven. Manifests and index are root files.
The 1024-owner test independently enumerates all 1030 prefixes/2049 payloads;
the atomic publisher additionally enforces the corresponding runtime limits.

Before source rendering or JSON serialization, reconcile the complete owner,
declaration and dependency inventories, reserve every source_byte_bound, and
add conservative checked metadata/index reservations. JSON string reservation
must account for escaping and repeated occurrences, not just input character
counts. Bound flat inventories without recursively expanding shared ancestry or
dependency graphs. Arithmetic overflow and unknown projection shapes reject.

One typed field traversal targets two separate sinks: the reservation sink adds
fixed syntax bytes, 6*UTF-8-length+2 per quoted string occurrence, 35 bytes per
quoted declaration ID, and 20 per unsigned numeric location. It creates no JSON
payload. The encoding sink escapes strings and checks its reserved ceiling on
every append. All owner and index reservations fit the aggregate budget before
any source rendering or JSON encoding begins. Manifest reconciliation reprojects
the exact retained immutable owner before rendering; private fields cannot be
replaced through a public API.

After rendering, exact file names/counts and actual aggregate bytes must match
the reserved inventory. Only then call the shared bounded atomic directory
publisher once. No intermediate owner output becomes the production result.

## Required proof

Compiler-to-description assertions cover public and private declarations, aliases,
docs and used dependencies. Mutation tests reject missing/replaced/duplicate
owners, declarations and manifest inventories. Exact/one-over resource boundaries
and arithmetic overflow reject. Relocation and graph-record reordering are
byte-identical. Production bundle native/privacy/failure/race proof remains D03.

The native fixture's independent observation format is not production JSON:
typed source/target/module/binding rows and ordered doc rows are compared against
every serialized public and private declaration, not just exports. The fixture
also requires its complete compiler Fn/Struct/Field ID set to equal the retained
description set. Mutation tests must demonstrate that omitted/private metadata
or changed targets/visibility/docs fail the oracle before privacy consumers run.
