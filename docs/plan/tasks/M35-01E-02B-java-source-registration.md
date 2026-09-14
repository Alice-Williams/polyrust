# M35-01E-02B — Java source registration coherence

- Status: complete
- Parent: [M35-01E-02](M35-01E-02-java-source-identity.md)
- Depends on: completed M35-01E-02A

## Goal

Bind RustSource declaration metadata to the containing typed Java crate package
before adding source record members and documentation attachments.

## Implementation contract

- A RustSource registered type, callable, interface method or static value is
  a declaration, never a body-local parameter/binding/scope origin.
- Its declaration, owning module, export root and restricted visibility owner
  belong to the RustCrate namespace's crate ID. Legacy Generated packages
  cannot carry RustSource registrations.
- One Rust declaration ID has one registered target declaration. Check all
  four registration categories together, not only duplicate names or types.
- Enforce the rule in the existing package verification hook, repeated during
  post-link certification. Keep non-Rust registrations unchanged.
- These are metadata coherence checks, not independent source-analysis proof.
  HIR lowering still runs only after the compiler's checked-analysis boundary.

## Definition of done and tests

- A registered RustSource target type verifies, links, certifies and renders in
  its crate namespace with a provenance-neutral header.
- Wrong package/owner/module/export-root/restricted-scope identities, body-local
  origins and duplicate IDs across each registry category reject.
- Equal names with distinct identities and repeated render determinism retain
  the existing Java syntax rules; metadata cannot bypass those rules.
- All legacy Java tests, Rust/Bazel linters and the full migration gate pass.
- Fresh independent review closes core findings.

## Scope boundary

Record member identity/type/access, export-graph and documentation coherence,
and compiler HIR mappings remain in the parent and later tasks. This slice
does not authenticate metadata against the compiler, promise public API
equivalence, or introduce source parsing or raw target text.

## Completion evidence

- Full Linux/Bazel migration gate ebc1dabe-ed52-4c2b-90e8-dfac164c80d4:
  343/343 tests. Because planning docs changed during that run, the unchanged
  tree was rerun: a7b5b5db-f206-4c76-9c27-e73a30d77325, 343/343 tests,
  413 targets, 23.019 seconds. Java unit/native suite contains 222 passing tests.
- Rust/Bazel lints and historical language/example tests remain enabled.
- Positive source fixture traverses unresolved verification, linking, resolved
  certification and three deterministic renders; source and synthesized
  origins produce identical target syntax with a provenance-neutral header.
- Every required identity mutation and registry category is covered. The
  duplicate-name negative declares two real classes, rather than relying on
  unused registrations to represent emitted code.
- Sol Extra High java_source_registration_review found no core or other issues,
  including the pre-link and repeated post-link package-hook integration.
- No source-analysis claim, disabled tests, commit or push in this checkpoint.
