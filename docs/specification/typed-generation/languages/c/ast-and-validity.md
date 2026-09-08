# C17 AST and validity contract

- Status: normative for M34A-11
- Language: C17; conservative closed subset, not every possible C program

The [closed grammar inventory](grammar-inventory.md) enumerates the exact
variant domain, payloads and contextual obligations for this contract.

## Type and reference categories

The type model distinguishes object types, function signatures and void return
types. Full declarators are derived structurally from these types, not stored
as an independent string that can disagree with a signature. Pointers retain
their pointee and its qualifiers; function pointers retain an exact prototype.
Arrays have nonzero constant bounds and complete object elements. Functions
cannot return arrays/functions, and parameter arrays/functions are represented
as their explicit adjusted pointer types. Empty parameter lists mean `(void)`,
never an unspecified-arguments declaration.

Nominal structs, unions, enums, typedefs, members, functions, locals, labels and
files have separate typed identities with origin and authoritative registration.
Ordinary identifiers (including typedefs/enumerators), tags, labels and
per-aggregate members use C's distinct namespaces. Text is a spelling, not an
identity. Protected keywords, leading-underscore reserved spellings, standard
header symbols and generated helper names cannot be captured by user names.
Portable names are deterministically allocated by identity rather than rejected
because a target happens to reserve their spelling.

`CExpr`, `CStmt`, `CDeclaration`, `CDefinition`, `CInitializer` and
`CFileItem` are separate Rust categories. A value expression records its exact
type and value category; an lvalue is not silently interchangeable with a
borrow or mutable place. Known standard operations are enums with complete
signatures, not calls manufactured from strings.

## Initially admitted target grammar

Objects: fixed-width signed/unsigned integers, Boolean, double, size_t,
Unicode/byte storage types, concrete nominal aggregates, qualified pointers,
constant arrays, typed function pointers and structurally designated initializers.
Control: blocks, declarations, assignments to verified local/owned places,
if/else, bounded generated loops, switches, returns and owned cleanup exits.
Target-only loops and cleanup jumps are permitted only with their verified
control/ownership contracts; they do not add generic mutable-loop capabilities.

No raw text/tokens, arbitrary pragmas, inline assembly, VLA/flexible arrays,
variadic or old-style functions, implicit declarations, untyped callable casts,
object/function-pointer interconversion, unrestricted pointer arithmetic,
setjmp/longjmp, volatile/atomic access, bitfields or inheritance emulation.
Adding any excluded construct requires an explicit AST variant and proof.

## Verification and certificate

The contextual checker independently walks stored children and authenticated
declaration references. It reconstructs local relations using the same closed
constructor/scalar rules, then compares the reconstructed structure with the
input. A cached type, copied brand, or caller-supplied inventory is not evidence.
The diagnostic-only local-structure entry point proves only those local
relations; it cannot render or manufacture a contextual certificate.

Subsequent passes compare the actual declaration/control occurrences against
the authoritative registry, derive lexical visibility and a private control-flow
graph, and intersect definite-initialization facts over reachable predecessors.
They inspect unreachable syntax too, but exited branches do not enter later
initialization joins. Ownership, arithmetic, call effects and linked file
ordering retain their separate mandatory checks. The final shared verifier
composes these passes; none is a public render-ready shortcut.

Complete-object checks cover registered allocation storage and actual object
read/write/member/index uses, not only declared variables. Pointer indexing
requires a complete element even when its address alone is requested. Plain
pointers to incomplete tags and address cancellation remain admissible.
Every expression's type form is checked too: an expression-only pointer to an
array cannot hide an incomplete array element. A function-pointer prototype may
name incomplete by-value aggregate parameters/returns; actual function
definitions still require complete parameter/return objects.
Syntax/type/lexical walks always inspect every child; initialization's separate
runtime-path traversal respects proven short-circuit and conditional selection.
An unselected read does not require initialization, but cannot hide invalid
syntax, references or types. Unknown conditions retain both possibilities.
Exact local-array paths include every admitted integer literal category and
authenticated enumerator value; recognizing those leaves is not an arithmetic,
bounds or ownership proof. The [origin matrix](symbols-and-files.md) separately
checks registration and actual definition ownership before linked-name checks.
Joins retain common containing-object initialization implied by different union
member writes, without inventing struct siblings or unwritten array elements.
Active-union-member safety remains a separate mandatory ownership proof.
Movable tags, functions and objects require an actual declaration in their
registered owner file; a definition in another file does not discharge that
obligation. A same-file definition may serve as its own declaration.

Local verification checks exact declarations/prototypes, scopes, duplicate
definitions, initialization, qualifiers, callable/member ownership, expression
types, returns, labels, switch case constants and complete types. Definition
ordering and linkage are rechecked after helpers and includes are resolved.
A goto cannot bypass initialization or cleanup; C labels precede statements,
not bare declarations. Case labels use checked constant values, not textual
inequality. The only deliberate compile-negative artifact carries a closed,
independently demonstrated invalid shape and cannot join production files.

Signed arithmetic, shifts, casts, indexing, division and memory operations
also require the explicit safety rules in the ownership/ABI specification.
Native compilation alone does not prove absence of undefined behavior.

Only the shared verify/link/certify sequence constructs
`RenderReadyPackage<CDialect>`. No public or crate-visible source bypass,
serialization, mutable proof view or unchecked-to-render overload is allowed.
Every accepted grammar/category and contextual rule needs positive controls,
rejected mutations and actual pinned compiler evidence before migration exit.
