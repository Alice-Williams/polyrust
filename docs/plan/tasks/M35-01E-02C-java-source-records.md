# M35-01E-02C — Typed Rust-source Java record members

- Status: complete
- Parent: [M35-01E-02](M35-01E-02-java-source-identity.md)
- Depends on: completed M35-01E-02B

## Goal

Represent immutable source record fields with explicit nominal/source identity,
without inventing CoreFieldId values or relying on name-only structural fields.

## Implementation contract

- Add JavaSourceFieldOrigin containing the owning Rust declaration and shared
  RustSourceOrigin field metadata. A distinct RustSource record-component enum
  variant retains that metadata; Core and runtime variants remain separate.
- Add a RustSource field-reference enum variant carrying GeneratedTypeId owner,
  RustDeclarationId field identity, JavaIdentifier spelling and JavaType.
- Verify a source component belongs to its registered RustSource nominal owner,
  has declaration-node provenance and matching crate/module/export-root identity,
  and is not a reused owner/field identity. All source record components must
  use source origins; source origins cannot attach to unrelated Core/runtime
  or unregistered structural owners.
- Resolve field references against that exact declared owner/component identity,
  name and type. Include nominal references in symbol/link/resource traversal.
- Preserve existing private nest access, immutable final-field assignment,
  constructor definite-assignment and exact constructor signature checks.
  Initial HIR construction will emit explicit canonical record constructors;
  constructor arguments are evaluated before field-order rearrangement.
- Rendering remains a projection of checked field names/types; no raw fallback.
- Keep source-field checking in a dedicated module and fixtures separate from
  tests. Do not expand already large expression verifier files with new suites.

## Definition of done and tests

- A private nested source record, explicit constructor and source-field read
  certify, compile with Java 21 -Xlint:all -Werror, and execute a scalar result.
- Wrong target/source owner, unregistered target owner, field ID, spelling, type, source node,
  mixed origin category, duplicate field identity and external private access
  reject. Missing/double initialization and mutation outside constructors reject.
- Name-only Structural access cannot bypass source identity, including reads
  and constructor assignments. Legacy/runtime Structural access stays supported.
- Positive source field cases cover int/bool; generated output is deterministic.
- Historical Core/runtime field tests, resource gates, full migration/lint/docs
  gate and fresh independent review pass without weakening checks.

## Scope boundary

This is checked target representation. It does not establish rustc provenance,
public record ABI, heap semantics or new interface/conformance capabilities.
Documentation attachment/escaping and full export metadata coherence follow
in the parent before HIR integration is declared complete.
GeneratedTypeId remains the existing package-local arena reference: this slice
rejects missing/wrong current owners, not two independently constructed equal
index values. Compiler-session/dependency authority is checked at those later
boundaries, not inferred from target syntax metadata.

## Completion evidence

- Full Linux/Bazel gate 2d43907d-cd2d-41cb-a115-0152d6ded240:
  343/343 tests, 413 targets, 218.818 seconds. Java unit/native suite:
  231 passed, zero failed/ignored. No test disabling or dependency additions.
- The private nested int/bool source record certifies, renders three identical
  times, compiles under pinned Java 21 with -Xlint:all -Werror, and executes
  boundary values through a separately compiled consumer. External construction
  fails with Java's private-access diagnostic.
- Identity, owner, type, origin, constructor, finality and private-access
  regressions pass. Review found a name-only Structural field access bypass;
  it was repaired and read/constructor-assignment regressions now enforce it.
- Original review closed; independent Sol Extra High
  java_source_records_fresh_review found no remaining findings after tracing
  all field consumers, accessor alternatives and post-link integration.
- New modules/fixtures remain focused and below 350 lines. Planning/docs and
  Buildifier follow-up 3048463e-4e00-4876-9b32-8c8c7204e4b1 passed 2/2.
- No compiler-session or full Rust translation claim; no commit/push yet.
