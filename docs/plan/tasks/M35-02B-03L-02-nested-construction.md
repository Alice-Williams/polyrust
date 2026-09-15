# M35-02B-03L-02 — Typed nested-record construction capability

- Status: planned
- Parent: [M35-02B-03L](M35-02B-03L-nested-owned-records.md)
- Depends on: M35-02B-03L-01
- Specification: [nested owned records](../../specification/typed-generation/languages/c/rust-nested-owned-records.md)

## Contract

Authenticate one canonical complete nested-record construction operation after
successful compiler analysis. The layout uses actual local nongeneric AdtDef,
DefId, GenericArgsRef, Ty and FieldIdx. A closed field-kind enum distinguishes
the exact standard Box<i32, Global> leaf from a nested record layout. This is
operation identity, not ownership-flow or whole-body admission.

## Definition of done and tests

- Canonical owner/node checks, complete field membership, compiler-resolved
  initializer identity/type and absence of adjustments; no spelling authority.
- Finite default-representation named-field records, no custom Drop or cycles,
  no generic arguments/parameters, tuple/unit/union/enum/scalar/reference fields.
- Pin separate depth and aggregate-field budgets in the specification before
  implementing traversal. Repeated nominal types in distinct sibling fields
  are not themselves a recursive cycle; every field occurrence counts.
- Root construction must contain a nested record; existing flat input remains
  its own capability. Expose declaration order separately from initializer
  evaluation order and retain full leaf types including allocator arguments.
- Add one optional consuming builder slot with an executable Mapping signature;
  missing, duplicate, wrong capability/context/output/input and private evidence
  fail exact compile-negative tests. Historical registrations remain compatible.
- Positive nesting/repeated nominal/reversed initializer cases and negative
  types/shape/budgets/custom allocator/forged node controls are non-vacuous.
- Full isolated gate, fresh independent review, recorded evidence and a dedicated
  commit/push. No body certificate or target heap output is enabled.
