# C17 catalogues, linking and source files

- Status: normative for M34A-11

The [platform and namespace contract](platform-and-proof.md) fixes macro
reservations, visible ABI name allocation, system-link requirements and
non-executable metadata ownership.

Closed known-type/function/constant enums own authoritative identifiers,
header origins, exact prototypes, qualifier and ownership contracts, allocation
effects and required platform properties. Generated references own their
declaration identity, origin and complete structural type. Catalogue metadata
and verifier signatures have one source; strings cannot synthesize calls.

The linker computes includes from actual referenced types/operations, including
runtime definitions and scalar typedefs. Local headers are typed file identities,
not caller-supplied include paths. Include guards are allocated from file
identity and rendered by a closed guard node. File dependency edges distinguish
forward-declarable tags, complete-by-value definitions, function prototypes and
external definitions. Pointer cycles may forward-declare; impossible by-value
cycles are rejected. All public headers compile standalone and in both inclusion
orders, twice, and from a separate consumer translation unit.

A namespace-aware allocator protects C keywords, standard identifiers and
generated helpers without restricting the generic portable-name API. Exact
ordinary/tag/member/label namespaces and C linkage are checked. External
definitions are unique across the complete package. Static declarations cannot
leak into a public prototype. A typedef name is never confused with its tag.

Public opaque layouts and private vtables live in implementation/private files.
Public header/source/runtime/test roles are enums, not tests on filenames.
Helpers expand to structural declarations/definitions and their typed dependency
graph. Complete-layout prerequisites must be acyclic; callable references may
form strongly connected components resolved through registered prototypes.
Specialization identity registration precedes body construction, using finite
visited-identity traversal. Program-specific lifecycle/vtable helpers belong
to Implementation, preserving the baseline Runtime-to-user dependency ban.
The final file/type/member/prototype inventory is checked against registrations
in both directions so mutually deleted evidence cannot hide a missing item.

The renderer prints already-resolved includes, guards, declarations and
definitions. It does not discover complete-type order, helpers or ownership
cleanup by scanning source. Exact import/placement/collision/cycle mutations
and separately linked multiple translation units are permanent gates.
