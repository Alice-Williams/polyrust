# M30-04C — Migrate TypeScript and derived JavaScript fragments

- Status: in-progress
- Depends on: M30-04B

## Current audit finding

TypeScript declarations are accumulated into one body string with file-level
runtime requirements. TypeScript runtime and Node shim sources bypass
`LanguageSourceFile`; JavaScript copies a separate complete runtime string.

## Definition of done

- Type, value, declaration, test, and runtime mappings return `Ecma` fragments
  with default/named/side-effect and type-only import data.
- JavaScript is derived from the same fragments by explicit type erasure; it
  does not maintain a parallel semantic runtime inventory.
- Runtime and Node shim source roles use closed source files and helper closure.
- `EcmaImport` validates module/symbol data and only the renderer spells imports.

## Required tests

- Exact matrices for value/type imports, side effects, Node test APIs, runtime
  helpers, and programs with no imports.
- A parity test proves TypeScript and JavaScript select the same non-type helper
  graph and observable behavior.
- TypeScript compile/typecheck, generated JavaScript execution, ESLint or the
  configured linter, conformance, and public consumers.
- Three-generation byte determinism for both outputs.
