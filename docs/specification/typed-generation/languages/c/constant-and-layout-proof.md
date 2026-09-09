# C17 constant and layout proof

- Status: normative for M34A-11-02D-01
- Owner: ownership/constants and ownership/layout
- Prerequisite: reconstructed contextual package, not caller-labelled constants

## Representation authority

The existing C scalar ABI remains authoritative for compatibility, promotion,
usual arithmetic conversion and the new closed width/representation categories.
Do not use Rust usize, host pointer size, enum layout or unchecked Rust integer
casts as the target model. Integer values retain an actual scalar type and a
range-checked mathematical value. Bool is 0/1; numeric conversion to Bool is
zero/nonzero normalization, not narrowing modulo two.

Integer conversions to unsigned types reduce modulo their target width.
Conversions to signed types require representability; implementation-defined
out-of-range signed conversion is outside the verified subset. Promotions and
usual conversions use the same rules as AST construction. Integer operations
check the promoted type: narrow operands may operate as Int, not their declared
storage width. Unsigned arithmetic wraps at that promoted width; signed
overflow, minimum/-1 division or remainder and zero divisors are rejected.

Shift counts are checked after independent promotions and must be nonnegative
and smaller than the promoted left width. Signed left shift requires a
nonnegative left value and a representable signed result. The pinned compiler
model uses arithmetic right shift for negative signed values; both compiler
probes must establish this implementation-defined choice before closure.
Bitwise operations retain the pinned two's-complement representation.

These C-specific distinctions follow WG14's
[C draft, 6.3.1.3/4 and 6.5.7](https://www.open-std.org/jtc1/sc22/wg14/www/docs/n1570.pdf).
In particular, the signed-left-shift rule is not copied from C++.

## Floating constants and conversions

Binary64 calculations use the specified noncontracting, nontrapping,
round-to-nearest/ties-to-even platform environment. Integer-to-double rounding
and negative signed right shift are independently probed; merely observing that
the host happens to use the same width is not proof of the target choice.
Numeric facts may distinguish NaN, infinities and signed zero, but cannot promise
a particular arithmetic NaN payload or use it as portable representation proof.
The original AST is retained for rendering; this analysis does not replace it
with host-formatted literals or folded executable text.

Float-to-integer conversion checks finite truncated values against the exact
target range before conversion. It must not rely on Rust's saturating cast
behavior, or compare against a rounded inclusive integer maximum. Bool conversion
uses zero/nonzero even for NaN. Floating environment and portable raw-bit
semantics remain additional runtime/mapping obligations in stages 05/07.

## Object layouts

Layout derives from actual complete registered members, never a stored caller
size or a generated spelling. Scalar, pointer and MaxAlign layouts follow the
ABI model. FILE has no admitted by-value layout. Enum layout requires its actual
complete enumeration. Typedefs retain authentication while resolving layout.

Arrays use checked nonzero-bound multiplication. Struct member offsets use
checked alignment round-up in declared order; tail padding uses the maximum
member alignment. Union members share offset zero and size is the aligned
maximum member size. Every sum, product and round-up is checked in the target
Size domain. By-value cycles and incomplete operands fail; recursive pointers
do not require pointee layout. Layout traversal uses explicit work/state so an
input graph is not mistaken for recursive execution or an already proved cycle.

A representable layout is not a compiler-capacity certificate: native object,
translation and frame budgets remain mandatory stage 04 checks. In particular,
Size representability alone does not admit an enormous object to rendering.

## Traversal and private facts

All-syntax checking validates every type form and every SizeOf/AlignOf operand,
including unreachable and unselected syntax. Evaluated constant arithmetic
respects LogicalAnd, LogicalOr and Conditional selection: an unselected unsafe
operation is not executed, but malformed types/identities remain rejected.
Static initializer leaves and assertion conditions are rederived from their
actual children. Assertions must evaluate nonzero. Address constants retain
their structural children; numeric index arithmetic is checked here while
pointer extent/lifetime safety remains a mandatory storage-analysis obligation.

Only private constructors create checked scalar/layout facts. Package-level facts
borrow the same immutable registry and files whose contextual checks succeeded;
consumers cannot substitute another package or attach a safe flag. Public
diagnostics, if exposed, return no facts or rendering permission. Combined
02D verification still owns runtime ranges, ownership, sequencing and calls.

## Proof

Require positive/rejected controls for every integer representation, promotion
pair, operator and conversion boundary; unsigned wrapping versus signed failure;
shift extrema; signed minimum divided/remainder by minus one; finite/nonfinite
float-to-integer boundaries; and known constant values.
Layout controls cover scalars, pointers, known objects, enums, aliases, arrays,
struct/union padding, incomplete/cyclic graphs and arithmetic capacity edges.
Package controls pair false/true assertions and selected/skipped unsafe constant
operations with unconditional malformed-child rejection. Native probes remain
repository-only oracles and permanent release-gate members, never production
source escapes or substitutes for the typed checks.
