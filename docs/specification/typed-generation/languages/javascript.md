# JavaScript derived-output specification

- Status: normative for M34A
- Target ID: `org.polyrust.javascript`
- Derivation toolchain: TypeScript 7.0.2, Node.js 24.20.0

## Inferred typed-program admission

JavaScript inherits the exact `TypedProgram<R>` admission proof from
`TypeScriptPlugin: SupportsAll<R>` and additionally requires the pinned
TypeScript-to-JavaScript derivation stage. It MUST NOT advertise an independent
semantic `Supports<F>` registry, because doing so could drift from the sole
TypeScript source program.

## 1. Architectural status

JavaScript is a derived output of the TypeScript plugin. It does not own:

- a semantic CoreIR lowerer;
- an executable JavaScript AST;
- a portable feature registry independent of TypeScript;
- an executable runtime-helper catalogue; or
- an executable renderer or executable template.

This prevents TypeScript and JavaScript behavior from drifting.

## 2. Input and output

Input is the verified, rendered TypeScript package plus the pinned TypeScript
compiler configuration. The derivation stage emits:

- compiled `src/index.js`;
- compiled `src/runtime.js` when present;
- compiled conformance/native tests;
- deterministic JavaScript package metadata; and
- optional source maps only if explicitly specified and deterministic.

No executable JavaScript file may be hand-edited or independently rendered.

## 3. Capability relationship

The JavaScript target mirrors the TypeScript runtime capability result after
type erasure. A feature may be:

- supported identically at runtime;
- rejected because it requires a TypeScript-only guarantee with no generated
  runtime enforcement; or
- supported by an explicit TypeScript-emitted runtime witness.

The derived target cannot claim a capability absent from the TypeScript
plugin.

## 4. Types and interfaces after erasure

TypeScript annotations/interfaces erase. Runtime values remain the generated
immutable/frozen representations. Interface dispatch is ordinary JavaScript
method dispatch over the TypeScript-generated implementation/witness.

No prototype inheritance, prototype mutation, mixin, or target-specific
JavaScript semantic rewrite is introduced after compilation.

## 5. Symbols and dependencies

Executable imports are emitted by the TypeScript compiler from resolved
TypeScript module references. The JavaScript packager may resolve metadata
paths/extensions but cannot infer or insert semantic imports.

Node/package dependencies are exactly the runtime-bearing subset derived by the
TypeScript resolver.

## 6. Runtime

`runtime.js` is compiler output from `runtime.ts`. Runtime helper marker,
identity, selection, and behavior originate in typed TypeScript AST.

A checked-in compiler-derived fixture may exist only as an exact derivation
artifact with a hash/parity test. It is never independently edited.

## 7. Rendering and packaging

Executable source uses no JavaScript renderer or template. It is produced only
by the pinned compiler from a certified TypeScript package. Typed metadata
models may use strict templates for `package.json` or documentation.

The metadata renderer cannot alter, concatenate, or wrap executable JavaScript.

## 8. Validation

The derivation verifier checks:

- exact pinned compiler identity/options;
- every expected TypeScript executable input has a derived JavaScript output;
- no unexpected executable file appears;
- no TypeScript source is shipped in the standalone JavaScript package;
- source hashes match a clean compiler invocation;
- helper/module topology matches the TypeScript package after erasure; and
- package entry points reference only derived files.

## 9. Success evidence

- Clean TypeScript compilation produces byte-identical JavaScript three times.
- A deliberate JavaScript edit fails the derivation hash test.
- A deliberate extra/missing source fails package validation.
- Prettier check is deterministic for compiler output.
- Standalone `npm test` runs without TypeScript tooling at runtime.
- Exact portable/native/conformance behavior matches TypeScript.
- Interface dynamic dispatch survives type erasure.
- Every historical port passes standalone JavaScript tests.

## 10. Migration exit

The JavaScript target passes only after its independent emitter/paired raw
source path is deleted and all executable bytes are demonstrably produced by
the pinned TypeScript compiler from typed TypeScript AST output.
