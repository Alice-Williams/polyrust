# TypeScript typed-generation specification

- Status: normative for M34A
- Target ID: `org.polyrust.typescript`
- Language/toolchain: strict ESM, TypeScript 7.0.2, Node.js 24.20.0

## Inferred typed-program admission

`TypeScriptPluginBuilder` MUST register one typed `TsCapabilityMapping<C>` for each
portable capability whose complete strict-TypeScript mapping has passed this
specification. Only an implemented builder slot derives
`TypeScriptPlugin: Supports<C>`. Typed generation accepts `TypedProgram<R>`
only under `TypeScriptPlugin: SupportsAll<R>`. There is no empty, profile-wide,
wildcard, or default support claim.

## 1. Scope and package

The plugin emits:

- `package.json`;
- `tsconfig.json`;
- `src/index.ts`;
- `src/runtime.ts` when helpers are selected;
- `src/conformance.test.ts`;
- `src/generated.test.ts`;
- `src/node-shims.d.ts` when required; and
- `tests/invalid-types.ts`.

The emitted TypeScript is the sole executable source for the derived JavaScript
target.

## 2. Capability strategies

The exhaustive TypeScript registry distinguishes native JavaScript semantics,
typed TypeScript-only declarations, and runtime emulation.

Fixed-width integer behavior, exact binary64 representation operations, Unicode
scalar validation, immutable collection behavior, and callable failures use
explicit native/emulated strategy enums. No mapping relies on JavaScript
coercion outside the declared typed API.

## 3. TypeScript AST

The dialect owns distinct:

- `TsType`;
- `TsExpr` and `TsPrecedence`;
- `TsStmt`;
- `TsPattern`/binding nodes;
- `TsDeclaration`;
- `TsClassMember` and `TsInterfaceMember`;
- `TsImportExport`;
- `TsFile`; and
- closed TypeScript grammar and formatting categories.

Type-only syntax is represented structurally, not by paired source strings.
Every node declares whether it survives JavaScript compilation when that
property is needed by derivation verification.

## 4. Type mapping

| CoreIR type | TypeScript representation |
| --- | --- |
| Unit | generated frozen unit singleton type/value |
| Bool | `boolean` |
| I32 | `number` constrained by fixed-width operations |
| I64 | `bigint` |
| F64 | `number` with exact-bit helpers where required |
| Char | validated one-scalar `string` |
| String | Unicode-scalar-valid `string` |
| Bytes | readonly copied byte representation |
| List<T> | `ReadonlyArray<T>` with copy-producing operations |
| Option<T> | readonly discriminated union |
| Result<T,E> | readonly discriminated union |
| Record | generated nominal readonly class |
| Tagged enum | readonly discriminated union |
| Interface | flat TypeScript `interface` and immutable implementation value |

Public collections and records cannot expose portable mutation.

## 5. Declarations and control

- Constants are exported/non-exported `const` values with frozen/copy-safe
  composite representations.
- Records are readonly classes or equivalent nominal declarations with
  deterministic branding only where needed.
- Enums/options/results use explicit discriminant properties.
- CoreIR matches lower to exhaustive target control with verifier-generated
  unreachable checks where useful.
- Every integer operation uses the registered fixed-width strategy.
- Exact F64 literals and representation transforms use typed `DataView`
  catalogue calls where native arithmetic is insufficient.
- Exceptions are not portable result flow.

## 6. Interfaces and composition

Portable interfaces lower to flat TypeScript interfaces with no `extends`.
Generated records explicitly satisfy their checked interface shapes. Multiple
independent conformance is allowed.

Interface values use normal object-method dispatch over immutable generated
objects. The target verifier proves the exact method set and signatures.
Composition uses readonly fields and explicit delegation; mixins, prototype
mutation, declaration merging, and automatic member forwarding are forbidden.

## 7. Symbols and imports

Closed catalogues cover:

- ECMAScript globals such as `Object`, `Array`, `BigInt`, `DataView`,
  `TextEncoder`, and `TextDecoder`;
- Node test/assert module symbols;
- generated and runtime symbols; and
- type-only versus value-bearing references.

Global symbols produce no imports. Node symbols derive exact `node:` imports.
The resolver distinguishes type/value namespaces, type-only imports, aliases,
relative module paths, re-exports, and collisions.

No mapping manually adds an import or relies on an import inferred from text.

## 8. Runtime helpers

`TsRuntimeHelper` is a closed enum. Helpers are TypeScript AST declarations.
Helper selection is exact and shared semantically with JavaScript derivation.

The runtime contains no checked-in executable TypeScript string and no
helper-specific template.

## 9. File and package policy

Typed file roles select package metadata, compiler configuration, public entry
modules, runtime modules, conformance/native tests, declaration shims, and
negative fixtures. The resolver owns exports, relative module paths, runtime
versus type-only edges, entry points, deterministic file names, and module-cycle
diagnostics. JavaScript output locations are reserved for compiler derivation.

## 10. Rendering

The TypeScript post-link checker certifies complete modules as an opaque
`RenderReadyPackage<TypeScriptDialect>`. It validates module/import/export
placement, value/type namespaces, declaration and binding scopes, interface and
class member shape, generic uses, narrowing/pattern forms, assignment targets,
statement context, and return coverage for the pinned TypeScript grammar.

The total TypeScript renderer structurally covers module files,
imports/exports, interfaces, classes, types, functions, methods, properties,
blocks, statements, expressions, patterns, literals, tests, and declaration
shims. It owns precedence, identifier/literal escaping, type-only spelling,
semicolon policy, and every fixed token through exhaustive AST matches. There
is no executable Handlebars template or token/source escape hatch.

The pinned TypeScript compiler and Prettier are independent acceptance/no-diff
oracles. Unformatted output MUST already parse and type-check; Prettier cannot
create the certificate or repair syntax.

## 11. Validation

The verifier checks:

- expression and type signatures;
- type/value namespace use;
- interface conformance and no interface inheritance;
- readonly/public mutation boundaries;
- discriminated union completeness;
- integer/F64 strategy use;
- module graph and export visibility;
- JavaScript-erasure classification; and
- absence of opaque code.

## 12. Success evidence

- AST category and typed-call compile-fail tests.
- Known global/module signature and exact import matrices.
- Flat interfaces, multiple conformance, first-class/nested interface values,
  static/dynamic dispatch, and explicit delegation.
- Strict negative type fixtures.
- Runtime helper absence/presence matrices.
- Three-generation determinism.
- Prettier check and second-run no-op.
- `tsc --noEmit` under strict settings.
- Native Node tests and every portable test.
- Compiler-derived JavaScript parity gate.
- Every historical port and canonical conformance vector.

## 13. Migration exit

The TypeScript plugin passes only after paired/raw `EcmaCode` source
construction is deleted, all executable source is typed `TsAst`, and the
derived JavaScript package passes its separate specification.
