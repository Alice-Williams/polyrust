# M35-02B — Authenticate structured ownership correspondence

- Status: planned
- Parent: [M35-02](M35-02-rustc-owned-values.md)
- Depends on: M35-02A

## Contract

Define a private compiler-session input for owned operations using existing
rustc identities/types. Specify how admitted HIR constructions, moves, borrows,
lexical exits and returns correspond to compiler drop obligations. Keep HIR as
the structured rendering input; do not translate MIR block edges into gotos.
Do not accept a variable name, debug annotation or source-span coincidence as
authority. If compiler data cannot uniquely establish a mapping, reject it and
document the restriction before implementing the target mapping.

## Definition of done and tests

- Normative closed shapes cover scalar Boxes, owned records, conditional/partial
  moves and function boundaries, with explicit staged exclusions where needed.
- Standard constructor/clone/drop operations use compiler identities and actual
  instantiated types; same-named user methods cannot acquire those capabilities.
- Executable capability slots have typed input/context/output signatures and
  compile-negative missing/wrong-registration tests.
- Shadowing, sibling scopes, temporaries, branches and early returns have
  positive correspondence and deliberately mismatched owner/place negative tests.
- No public safe API fabricates a checked ownership input or bypasses successful
  compiler analysis; invalid/unsupported cases leave output absent.
- Fresh review and the existing C/Java/native/lint gates pass unchanged.
