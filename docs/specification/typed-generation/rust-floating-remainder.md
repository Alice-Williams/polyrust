# Checked binary64 truncating remainder

- Status: implemented and verified for the bounded checked f64 source contract
- Plan: [02O](../../plan/tasks/M35-03A-02O-floating-remainder.md)

## Semantics

Admit only built-in f64 %. For finite operands with nonzero divisor, use
r = a - q*b where q is the exact mathematical quotient truncated toward zero.
This is a semantic definition, not a permitted floating division/multiply/
subtract implementation: rounded division can overflow or choose the wrong q.
The finite remainder is exactly representable. Its magnitude is smaller than
the divisor and its sign, including zero, follows the dividend.

NaN operands, infinite dividend and zero divisor produce NaN. A finite dividend
with infinite divisor is returned unchanged. NaNs are compared by category;
payload/sign, errno, exception flags/traps and foreign floating environment
changes are outside the pinned supported profile.

Rust's reference specifies truncating remainder; Java's floating % rules use
the same quotient convention rather than IEEE nearest-quotient remainder.
Sources: [Rust operators](https://doc.rust-lang.org/reference/expressions/operator-expr.html),
[Java21 remainder](https://docs.oracle.com/javase/specs/jls/se21/html/jls-15.html#jls-15.17.3).

## Typed layers

A private RemainderInput retains canonical source/ordered child identities and
is reconstructed only from original rustc typechecking evidence. A separate
FloatingRemainder executable builder slot owns its mapping; adding it must not
silently widen the existing four-operator FloatingArithmetic contract.
Both mappings finish/materialize the left operand before beginning the right.
Target certification retains recursive authority, numeric and resource checks.

[C](languages/c/rust-floating-remainder.md) and
[Java](languages/java/rust-floating-remainder.md) define the target nodes.
No copied runtime, custom remainder helper or raw source fragment is permitted.

## Proof

Use exact rational/integer expectations plus independently specified category
rules and goldens. Compare pinned Rust, then hand-built certified C/Java targets,
then actual Rust-source packages. Include huge quotient/exponent gaps, signed
zeros, subnormals, normal boundaries, infinities/NaNs and original call traces.
Wrong remainder family, rounded quotient, swapped or duplicated operands and
lost zero sign must be observed. Gate unsupported shapes atomically.
