# M35-01E-04 — Certified Java dependencies and crate bundles

- Status: complete
- Parent: [M35-01E](M35-01E-java-rustc-retrofit.md)
- Depends on: M35-01E-03

## Goal

Preserve separate compiler-checked crate APIs in Java without copying foreign
bodies or accepting descriptive manifests as proof.

## Implementation

Execute the following checkpoints in order, keeping implementation and proof
modules focused rather than adding a monolithic Java graph adapter:

1. [M35-01E-04A](M35-01E-04A-java-dependency-certificates.md): immutable owner APIs.
2. [M35-01E-04B](M35-01E-04B-java-imported-callables.md): consumer bindings and qualified calls.
3. [M35-01E-04C](M35-01E-04C-java-compiler-graph.md): compiler-authenticated foreign joins.
4. [M35-01E-04D](M35-01E-04D-java-bundle-publication.md): bounded atomic bundles.

- Add immutable dependency API/function handles derived only from certified
  Java packages; import exact signatures with consumer-owned references.
- Extend Java's dependency callable and qualified-name categories for these
  handles. A typed dependency spelling policy selects fully qualified Java static
  calls; these require no Java import directive. Existing JDK imports still derive
  from actual typed uses. Do not emit unused static imports or invent aliases to
  work around the repeated Generated facade name across crate namespaces.
- Join rustc foreign DefId/loaded-crate identities to the exact owner certificate
  through the existing checked graph driver.
- Validate the complete graph, render bounded files and publish atomically with
  no replacement, using the established publication mechanism.
- Serialize descriptive aliases/docs/ownership/import inventory, never proofs.

## Definition of done and tests

- Separate/diamond/repeated-alias/unused dependency graphs produce one owner
  implementation per crate and no copied foreign bodies.
- Wrong owner/session/signature/member certificate, private export, missing
  dependency, collision and graph/resource overflow reject before publication.
- Metadata agreement, declared source/doc invalidation, record reordering and
  physical relocation retain existing guarantees.
- Native Java consumers compile/run, illegal private consumers reject, atomic
  publication tests pass; full gates and fresh broad review are clean.

## Closure

E04A/B/C/D are complete. D03's final full gate
`49c0e966-89a7-4dc4-8b25-640149f83724` passed all 370 tests across 470 targets,
and fresh review returned no core findings. Complete typed/compiler metadata,
separate native Java ownership, deterministic production bundles and atomic
publication are proved for the admitted subset. E05 owns final migration closure.
