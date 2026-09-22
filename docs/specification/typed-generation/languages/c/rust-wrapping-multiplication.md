# Rust wrapping multiplication in C17

- Status: target foundation and checked compiler-source integration complete
- Contract: [shared](../../rust-wrapping-multiplication.md)

## Typed lowering

On the already validated C platform profile, materialize original signed operands
left-to-right. For matching S=I32/I64 and U=U32/U64, materialize u=U(left)*U(right).
The effective arithmetic type must remain unsigned at the intended width; do not
assume typedef spelling alone avoids integer promotions on an unsupported platform.
Reconstruct through the existing typed conditional:

    u <= U(MAX) ? S(u) : -1 - S(~u)

Each conversion is in range on its path; the signed subtraction is representable.
Normalize promoted signed I32 reconstruction to exact I32. Never multiply signed
operands directly or cast an out-of-range unsigned product to signed. This is our
lowering design using [C draft N1570 6.2.5/6.3.1.3/6.5.5](https://www.open-std.org/jtc1/sc22/wg14/www/docs/n1570.pdf).

## Certification and proof

Admit only same-width internal unsigned Multiply and visit both child subtrees.
Existing path-sensitive numeric and ownership/resource proofs remain active.
Products may wrap; that fact cannot establish nonwrapping allocation/extent
evidence. No trusted formula whitelist or public unsigned source ABI is added.

Test actual AST shapes, wrong guards/conversions, exact type/platform boundaries
and MayWrap transfer. Separate strict GCC14/Zig O0/O2 producers/clients, standalone
headers and GCC UBSan agree with the independent oracle. Compiling safe wrong
operation, saturation, narrowing and disconnected-operand controls must disagree.
Dependencies derive from typed symbols, without runtime or math-library support.

The completed target proof covers 34,546 pairs, both original owners, strict
GCC14/Zig O0/O2, standalone headers and UBSan. Six new focused tests and all
948 release/lint targets pass with clean independent review. Test-only saturation
and narrowing controls are disposable mutations, not generated runtime support.
