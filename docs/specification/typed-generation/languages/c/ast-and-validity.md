# C17 AST and validity contract

- Status: normative for M34A-11
- Language: C17; conservative closed subset, not every possible C program

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
