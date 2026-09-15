# M35-02B-03H — Authenticate Boxes containing scalar records

- Status: complete
- Parent: [M35-02B-03](M35-02B-03-structured-owned-places.md)
- Depends on: M35-02B-03G-02
- Specification: [boxed scalar records](../../specification/typed-generation/languages/c/rust-boxed-scalar-records.md)

## Contract

Add the other record ownership direction: one standard Box owning a local
named-field record of i32/bool values. This is distinct from G's record owning
several Boxes. Reuse canonical compiler identities and the executable capability
contracts; do not infer ownership from record or method spellings.

Begin with [H-01 — Constructor identity](M35-02B-03H-01-boxed-record-construction.md).
Continue with [H-02 — Whole-body correspondence](M35-02B-03H-02-boxed-record-correspondence.md)
after inspecting the pinned payload pointer/field/drop representation.

## Definition of done and tests

- Authenticate standard Box construction for one closed scalar-record payload,
  preserving the payload declaration, instantiated type and field identities.
- Retain source record construction and field evaluation order, whole Box moves,
  selected scalar field reads and final Box cleanup through typed evidence.
- Distinct nominal records with identical fields cannot substitute for each
  other. Every constructor, payload producer, move, read and drop is accounted.
- Positive i32/bool-field, alias/renaming, local-move and explicit-return cases
  have independent source/typed-place/cleanup negative controls.
- Reject nested/owned fields, generic or external records, custom Drop/repr,
  updates, arbitrary calls, borrows, unwind and ambiguous provenance.
- Preserve prior scalar Box and partial-record proofs, C/Java native/lint gates.
  Fresh review, isolated exact-tree tests and documented evidence precede push.

This compiler-only increment does not enable target heap generation or close
remaining conditional initialization, clone or function-transfer obligations.

## Closure evidence

H-01 closes separate constructor/type identity and executable registration.
H-02 closes the bounded scalar-record producer, payload transfer, whole Box
move, explicit field-read and root cleanup/return correspondence. Its exact-tree
gate passed 450 tests across 577 targets; both original and fresh independent
reviewers found no remaining core defects after the documented exit-evidence
and negative-test repairs. Each task records its own checkpoint evidence.
This closes only the stated compiler-side Box-of-scalar-records form.
