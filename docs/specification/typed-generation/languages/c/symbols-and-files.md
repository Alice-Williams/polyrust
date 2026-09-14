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

## Structural origin and file-role matrix

02C checks this closed definition-owner matrix before name allocation. GP/GS
are generated public header/source, RP/RS runtime public header/source, PH a
private header and TS test source. A dash rejects the combination.

| Origin | GP | GS | RP | RS | PH | TS |
| --- | --- | --- | --- | --- | --- | --- |
| CoreDeclaration except Test | yes | yes | - | - | yes | - |
| CoreDeclaration Test | - | - | - | - | - | yes |
| CoreExpression | - | yes | - | - | yes | yes |
| RustSource metadata | yes | yes | - | - | yes | - |
| Runtime synthesis | - | - | yes | yes | yes | - |
| OwnershipAdapter / InterfaceAdapter synthesis | yes | yes | - | - | yes | yes |
| EvaluationTemporary synthesis | - | yes | - | - | yes | yes |
| TestHarness synthesis | - | - | - | - | - | yes |
| PlatformAssertion synthesis | yes | yes | yes | yes | yes | yes |

Check both the canonical registration owner and actual definition file.
Nested parameters/scopes/locals/controls/allocation identities use the actual
function body file, not its public prototype header. Aggregate members use
the actual layout file. Merely referencing a symbol, including through a
forward declaration or prototype, does not transfer its ownership to that file.
Private headers may serve either family; test helpers may specialize adapters.
This matrix checks structural consistency of the supplied origin, not its truth:
exact Core IDs, test-expression provenance and legitimate specialization are
authenticated by checked Core mappings. Stage 03 separately checks the linked
Runtime-to-user dependency ban, test-only isolation and public/private leaks.

The renderer prints already-resolved includes, guards, declarations and
definitions. It does not discover complete-type order, helpers or ownership
cleanup by scanning source. Exact import/placement/collision/cycle mutations
and separately linked multiple translation units are permanent gates.

## Structural dependency discovery (M35-01B)

`file_dependencies` authenticates the complete source-file inventory against
`CFrozenRegistry`, then exhaustively visits declarations, definitions, statement
bodies, initializers and expression children. It returns one immutable
`CFileDependencies` per file, ordered by stable file key. This is dependency
analysis, not a RenderReadyPackage or a substitute for the remaining verifier,
linker, visibility, resource and ownership checks.

The result contains closed standard-header and system-library sets plus actual
registered tag, typedef, function and object references. No spelling lookup,
source scan, hard-coded prelude list or user-supplied include list participates.
Same-file references remain in the result: later placement must discharge them
using the real declaration inventory. Indirect-call contract witnesses are proof
metadata; they do not create an extra direct-call dependency. The actual
function-pointer signature owns indirect-call alias and complete-type edges;
a compatible proof witness's differently spelled aliases are not dependencies.

| Typed use | Required declaration evidence |
| --- | --- |
| Struct/union behind a pointer or in a prototype | Forward tag declaration |
| Stored/initialized/by-value aggregate, member access, sizeof/alignof | Complete definition |
| Pointer to a fixed array | Complete element definition, despite the pointer |
| Enum reference, including behind a pointer | Complete enum definition (C17 has no forward enum declaration) |
| Typedef use | Exact alias declaration and the underlying use's prerequisites |
| Function definition/call | Complete by-value parameters/result; pointers retain their weaker prerequisites |

Requirements merge monotonically: complete-definition use cannot be weakened by
a later pointer-only use. Signature discovery uses declared parameter/return
types so canonical compatibility does not erase alias dependencies. Nominal
members are inspected at their declaration, rather than recursively expanding
referenced layouts and looping through recursive pointer graphs.

The scalar spelling contract uses `_Bool` and numeric boolean literals, hence
does not need `stdbool.h`. Exact-width scalar spellings require `stdint.h`,
`size_t` requires `stddef.h`, and native char/int/double need no header. Known
object, constant and callable identities own their header mapping. Math link
requirements come from the callable catalogue, not from a `math.h` string test.
These are structural prerequisites, not a promise of globally minimal includes.

Tests cover the complete scalar matrix, constants/library objects, nested
pointers, array completeness, aliases in prototypes, expression-only constants
and math calls, foreign-registry rejection, deterministic file ordering and
dependency removal. The compiler-backed provenance fixture additionally checks
the actual HIR-lowered C tree's include and nominal requirements. Public-header
consumer compilation and linked-file mutation proof remain integration gates;
discovery alone does not complete them.
