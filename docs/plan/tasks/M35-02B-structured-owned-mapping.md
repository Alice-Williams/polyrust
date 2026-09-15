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

## Compiler-to-target handoff audit

K-02 completed the last listed operation family. Before closing this parent,
audit these obligations against the concrete child contracts, rather than
treating the union of accepted fixtures as arbitrary Rust support:

| Obligation | Existing evidence |
| --- | --- |
| Standard operations and typed registrations | B-01 constructor, G-01 record, H-01 boxed scalar-record, I-01 local call and K-01 clone inputs use compiler identities/types with executable slots and exact compile-negative controls |
| Whole body, moves and source scopes | B-02 plus 03A/B/C cover parameter-rooted owner chains, tail-nested scopes, shadowing, multiple owners and explicit returns |
| Sibling branch scopes and early exits | 03D/E retain canonical true/false branch or early-arm/continuation identities and prove both paths, guards and actual cleanup |
| Conditional transfer and cleanup | 03F authenticates selected owner transfers, compiler drop decisions and both complete paths |
| Partial records and temporaries | 03G-02 authenticates compiler initializer staging, declared field identities, partial moves and remaining-field cleanup |
| Boxed scalar payload records | 03H-02 authenticates aggregate construction, selected field reads and enclosing-owner cleanup |
| Function boundaries | 03I-02 authenticates two-function producer/consumer/relay graphs and matched caller/callee ownership, not signature-only effects |
| Clone and retained exit projections | 03J and 03K-02 expose canonical exits and exact operation/read/drop/Return locations; clone original and result remain separate owners |

The admitted sibling scopes are canonical conditional arms; this is not support
for arbitrary sequential non-tail sibling blocks. Admitted temporaries include
actual aggregate initializer staging, scalar argument staging and shared clone
borrow/reborrow staging; arbitrary temporary receivers remain unsupported.
Branches, partial records, boxed scalar records and call graphs have separate
closed readers. Their arbitrary nesting/combinations are not implicitly admitted.

No successful probe enables heap output. C runtime/allocation-failure policy,
target ownership admission and native event/sanitizer proof remain M35-02C/D;
Java needs its own later mapping. Custom Drop/allocators, unwinding, unsafe/raw
Box, leak/forget, String/Vec, recursive calls, arbitrary borrows and generic or
dynamic owned calls remain excluded. The next mapping must diagnose any source
shape or target obligation it cannot certify, without weakening these contracts.

The independent handoff audit found a real remaining obligation: G explicitly
requires nested owned records and conditional partial initialization/moves
before this parent closes. H and I discharged the other listed G follow-ups,
but the flat-record and scalar-selection readers do not discharge these two.
The table above describes existing evidence, not completion. This parent and
03 remain in-progress. Complete [03L](M35-02B-03L-nested-owned-records.md) and
[03M](M35-02B-03M-conditional-owned-records.md) before repeating the closure
audit. Do not erase these requirements by reclassifying them as optional source
combinations; C preparation does not authorize premature mapping admission.
