# M35-02B — Authenticate structured ownership correspondence

- Status: in-progress
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

## Ordered implementation

1. [M35-02B-01 — Constructor identities and executable binding](M35-02B-01-box-constructor-identities.md).
   Authenticate individual `Box<i32>` construction operations; this is not yet
   HIR/MIR operation correspondence or whole-body ownership admission.
2. [M35-02B-02 — Straight-line owned-place correspondence](M35-02B-02-linear-owned-places.md).
   Establish a closed straight-line HIR-operation/MIR-place relation, rejecting
   ambiguity. Names/debug information/spans alone cannot select a place.
3. [M35-02B-03 — Structured ownership correspondence](M35-02B-03-structured-owned-places.md).
   Extend that relation to structured scopes, branches, partial moves and
   function boundaries with typed mismatch controls before C mapping cutover.
