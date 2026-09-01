# M30-03 — Build the Java runtime helper graph

- Status: complete

## Goal

Replace the monolithic Java runtime body and fixed import inventory with the
transitive closure of helpers required by the checked program.

## Definition of done

- Each Java runtime helper node owns its declarations, structured imports, and
  dependencies on other helper nodes.
- Checked-program feature analysis selects roots; deterministic graph closure
  selects and orders all required helpers.
- JSON/evaluator infrastructure, numeric helpers, Unicode, bytes, collections,
  contracts, and portable-test support are independently selectable where their
  semantics allow it.
- The fixed `for import in [...]` runtime inventory is deleted.
- Cycles, missing helper IDs, and duplicate declarations produce diagnostics.

## Tests

- Unit tests for closure, stable topological order, deduplication, cycles, and
  missing dependencies.
- Minimal and one-feature-at-a-time Java programs prove exact helper and import
  presence/absence.
- All current Java native, conformance, negative, and public-consumer tests.

## Completion evidence

- `Runtime.java` is parsed into stable, ordered helper fragments; marker lines
  are build-time metadata and never enter generated source.
- Common evaluator infrastructure, the checked/wrapping integer family, and
  UTF-8 encode/decode support are separate graph roots. Each fragment carries
  the structured imports used by that fragment.
- Numeric roots come from checked capability evidence. UTF-8 roots use the
  shared target-independent semantic IR visitor, because the broader `Bytes`
  capability is intentionally insufficient to infer UTF-8 support.
- The former fixed `for import in [...]` inventory is deleted. Graph
  construction and resolution surface duplicate, missing, cyclic, malformed,
  and mismatched marker failures as generation diagnostics.
- Empty, registration, numeric-only, and UTF-8-only tests prove positive and
  negative helper/import selection. Shared graph tests prove stable ordering,
  transitive closure, deduplication, missing-root/dependency diagnostics, and
  cycle diagnostics.
- `//crates/backend-java:all`, Java 21 native/conformance/public-consumer
  compilation, `//:rustfmt_test`, `//:rust_clippy_test`, and
  `//:buildifier_test` pass in the Linux development container.
