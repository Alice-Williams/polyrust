# M35-03A-02S-02 — C wrapping-multiplication foundation

- Status: planned
- Parent: [02S](M35-03A-02S-wrapping-multiplication.md)
- Depends on: [oracle](M35-03A-02S-01-multiplication-oracle.md)
- Specification: [C17](../../specification/typed-generation/languages/c/rust-wrapping-multiplication.md)

## Contract

Admit exact same-width internal U32/U64 Multiply through the existing shared
profile. Preserve signed public signatures and certified guarded reconstruction.
Unsigned modular overflow must retain MayWrap/loss provenance, never allocation
extent or nonwrapping-size evidence. Existing numeric checks remain authoritative.

## Definition of done and tests

Positive both-width AST/native examples pass; mismatched widths, unsupported
operators, signed-overflow forms and malformed reconstruction reject. Both child
subtrees are visited. Strict separate GCC14/Zig O0/O2 compilation, standalone
headers and UBSan agree with the oracle. Safe compiling addition, saturation,
narrowing and disconnected-operand faults are detected at each width. Direct
overflow-transfer tests protect MayWrap provenance. Preserve addition/subtraction
bytes, full gate and independent review; no compiler-source admission yet.
