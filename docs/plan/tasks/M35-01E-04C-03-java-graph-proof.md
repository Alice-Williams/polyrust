# M35-01E-04C-03 — Independent Java compiler graph proof

- Status: complete
- Parent: [M35-01E-04C](M35-01E-04C-java-compiler-graph.md)
- Depends on: M35-01E-04C-02

## Implementation contract

- Observe actual checked graph certificates through a test-only adapter hook;
  keep production check mode non-publishing. Tests may render certificates into
  their own temporary directories for independent native verification.
- Compile each owning Generated.java exactly once at its canonical path with
  Java 21 --release 21 -Xlint:all -Werror, then separately compile handwritten
  consumers. Compare boundary/seeded cases with Rust reference execution.
- Check complete source export/doc and used-owner inventories, aliases, unused
  declarations, distinct same-spelled owners and absence of copied foreign bodies.
- Exercise wrong owner/declaration/signature substitutions and altered loaded
  metadata/source/docs. Reorder records and relocate inputs; compare exact bytes.

## Definition of done and tests

- Positive graph/native cases pass and every negative mutation rejects for its
  intended reason, without publishing a partial production result.
- Existing source-agreement and C regressions pass with the full migration gate.
- A fresh independent Sol Extra High review has no unresolved core findings;
  evaluate all feedback explicitly and repeat fresh review after core repairs.
- Record exact gate IDs and test counts, then mark parent C complete. E04D
  remains required before any claim of atomic production Java crate bundles.

## Completion evidence

- Native graph proof passes in `0d461063-e5cb-4eb3-8a1c-8bd991170168` and the
  expanded `ab2b5dbb-2a2e-47c4-aa72-82f528ddc4e4` gate. The existing four-crate
  Rust/C fixture is independently compiled as Rust and generated Java. All
  8,204 inputs x 16 outputs agree, including observable nested call order.
- Java owners compile separately, once each, followed by a handwritten consumer
  using Java 21 --release 21 -Xlint:all -Werror and no implicit source discovery.
  Private helpers reject; aliases, source docs, exact owner references and
  byte-identical record reordering/physical relocation pass.
- Compiler-owner/declaration/signature fault-injection adapters reject. Exact
  loaded metadata swaps, changed defining keys and changed source/docs reject,
  including unused dependencies. No failed production check publishes output.
- Full migration gate `28c8aacc-5c42-4b9b-b863-eb077308034b` passes all
  365 tests across 460 targets. No tests disabled.
- Fresh read-only Sol Extra High review `java_compiler_graph_review` found no
  actionable correctness defects or material proof gaps. It examined compiler
  identity/signature joins, complete graph certification, local-only lowering,
  source-order calls, mutation rejection, native privacy/docs and deterministic
  output. No findings required rejection or deferral. Production publication
  remains E04D's explicit scope, not an omitted E04C requirement.
