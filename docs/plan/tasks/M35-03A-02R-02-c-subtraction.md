# M35-03A-02R-02 — C wrapping-subtraction safety foundation

- Status: planned
- Parent: [02R](M35-03A-02R-wrapping-subtraction.md)
- Depends on: [oracle](M35-03A-02R-01-subtraction-oracle.md)
- Specification: [C17](../../specification/typed-generation/languages/c/rust-wrapping-subtraction.md)

## Contract

Admit exact internal U32/U64 subtraction in the certified dependency profile.
Use existing unsigned arithmetic flow and guarded signed reconstruction, without
special trusted formula recognition. Do not widen source/public signature types.

## Definition of done and tests

Typed subtraction/operand/result shape and recursive-child tests pass. Missing,
reversed and wrong-value guards reject unsafe signed conversions; direct signed
overflow still rejects. Modular loss cannot become nonwrapping extent evidence.
Original import authority, depth/resources and dependency-derived headers hold.
Separate strict GCC14/Zig producers/clients at O0/O2 and GCC UBSan agree with the
independent oracle. Compiling wrong-operation, reversed-result and narrowing
faults are detected. Full gate, old-output/WIP preservation and clean fresh review
precede the separate commit/push. No compiler source admission in this checkpoint.
