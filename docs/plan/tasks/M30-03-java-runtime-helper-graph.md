# M30-03 — Build the Java runtime helper graph

- Status: in-progress

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
