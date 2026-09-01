# M30-02 — Migrate Java lowering to fragments

- Status: complete

## Goal

Use Java as the first proof that every flat mapping can own its syntax and
requirements without a parallel declaration scan.

## Definition of done

- Java type, literal, expression, declaration, test, and compilation-unit
  lowerers return `JavaFragment` values.
- `JavaImport` stores a validated qualified name and import kind, never a source
  line.
- `Generated.java` is composed from declaration fragments.
- The record/enum import pre-scan and all equivalent manual synchronization are
  removed.
- Only `JavaRenderer` spells `import ...;`.

## Tests

- A feature matrix toggles records, enums, lists, maps, contracts, tests, and
  empty modules independently and asserts exact imports.
- Mutation/regression tests fail when an import requirement is removed from a
  fragment that emits dependent syntax.
- Generated Java formatting, compilation, portable tests, and public consumer.
