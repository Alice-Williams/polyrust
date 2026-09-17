# Rust wrapping negation in C17

- Status: target foundation implemented (M35-03A-02H-01); compiler integration planned (02H-02)
- Contract: [shared](../../rust-wrapping-negation.md)

## Typed mapping

Preserve exact I32/I64 storage. Materialize the source operand into an immutable
local x, then construct existing typed comparison/conditional/negation nodes:

    x == MIN ? MIN : -x

This is a mapping formula, not renderer text. Use checked signed literals,
CUnaryOperator::Negate and the typed conditional constructor. Normalize the
C comparison's Int result to Bool before constructing that conditional. C promotes I32 to
Int; normalize the final result back to I32 as in the existing bitwise mapping.
I64 stays I64. MIN is never actually negated.

The distinction is necessary because an unrepresentable signed result is not
defined wrapping arithmetic in C. The conditional evaluates only the selected
value; see [C draft N1570, 6.5, 6.5.3.3 and 6.5.15](https://www.open-std.org/jtc1/sc22/wg14/www/docs/n1570.pdf).

## Certificate and dependency obligations

Admit these exact scalar AST categories in bounded source and local-call
traversals; walk condition, both branches and the negated operand. Structural
admission is not numeric proof. Existing path-sensitive numeric-flow checks
must establish that the Negate operand excludes its minimum on every reachable
path. Missing, wrong-value and reversed guards must not certify.

Keep original callable authority, source budgets, stack/call-height evidence and
dependency-derived headers. No runtime helper, hidden support header, signed
overflow compiler flag, out-of-range unsigned-to-signed cast or widening-to-float
is allowed as a substitute. Signatures and public manifest schemas stay unchanged.

## Required evidence

Typed guard/normalization assertions, unsafe guard mutations and boundary native
execution with GCC/Zig O0/O2 plus GCC undefined-behavior sanitizer. Producers and
clients compile separately, including standalone public headers. Compare to
independent signed modular truth. No source admission changes in target-only step.
