# M30-04C — Migrate TypeScript and derived JavaScript fragments

- Status: complete
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

## Completion evidence

- One `EcmaCode` mapping traversal produces paired TypeScript and
  type-erased JavaScript syntax while carrying structured imports through
  nested types, declarations, implementations, and portable tests.
- `EcmaImport` has validated private semantic variants for default, named,
  type-only, and export-all dependencies. The renderer alone spells import and
  export directives; invalid modules and symbols are rejected.
- Runtime TypeScript and compiler-derived JavaScript share the same strict
  marker layout and helper graph. Exact intrinsic roots select paired helper
  declarations and dispatch cases; minimal and one-feature matrices prove
  positive/negative closure in both dialects.
- Runtime, index, test, conformance, negative-test, and Node-shim source roles
  are closed `LanguageSourceFile` values. Raw source-role bypasses and inline
  import types are absent.
- Three-generation determinism, Prettier, strict `tsc`, Node TypeScript and
  standalone JavaScript tests, runtime derivation, and all 130 tests in
  `//examples/real-world/...` pass.
