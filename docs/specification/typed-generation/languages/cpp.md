# C++20 typed-generation specification

- Status: normative for M34A
- Target ID: `org.polyrust.cpp20`
- Language/toolchain: C++20 through the hermetic Zig SDK, with GCC 14.2.0
  sanitizer verification

## Inferred typed-program admission

`CppPluginBuilder` MUST register one typed `CppCapabilityMapping<C>` for each
portable capability whose complete C++20 mapping has passed this specification.
Only an implemented builder slot derives `CppPlugin: Supports<C>`. Typed
generation accepts `TypedProgram<R>` only under
`CppPlugin: SupportsAll<R>`. Native and emulated mappings both require an
executable registration; there is no empty, wildcard, or default support claim.

## 1. Scope and package

The plugin emits self-contained public headers, implementation/runtime source
files where required, native/conformance tests, negative compilation fixtures,
and build metadata. A separate translation unit MUST consume the public header
without relying on include order or private definitions. The package MUST have
no undeclared third-party runtime dependency.

## 2. Capability strategies

The exhaustive C++ registry distinguishes native value semantics from checked
integer, exact float-bit, Unicode scalar, tagged result/option, immutable
collection, and type-erased interface strategies. It records allocation and
failure behavior explicitly.

Exceptions, undefined behavior, pointer identity, implicit numeric conversion,
RTTI, and implementation inheritance MUST NOT approximate portable semantics.
Unsupported constructs fail before C++ AST creation.

## 3. C++ AST

The dialect owns:

- `CppType`, cv/ref/pointer qualifiers, template arguments, and declarators;
- `CppExpr` and `CppPrecedence`;
- `CppStmt`, `CppBlock`, `CppCase`, and initialization forms;
- `CppDeclaration`, `CppDefinition`, `CppMember`, and `CppHeritage`;
- `CppFile`, `CppPackage`, typed header/source placement, and grammar
  categories; and
- closed enums for operators, casts, storage, visibility, special members,
  call forms, includes, attributes, and literals.

Ownership-bearing smart pointers and type-erasure records are explicit AST
forms/known types. No executable C++ source string enters the AST.

## 4. Type mapping

| CoreIR type | C++20 representation |
| --- | --- |
| Unit | generated empty value type |
| Bool | `bool` |
| I32 / I64 | `std::int32_t` / `std::int64_t` plus checked helpers |
| F64 | `double` plus `std::bit_cast`/registered raw-bit helpers |
| Char | validated Unicode scalar stored as `char32_t` |
| String | validated UTF-8 value wrapper over owned `std::string` |
| Bytes | immutable owned value wrapper over `std::vector<std::uint8_t>` |
| List<T> | immutable owned wrapper over `std::vector<T>` |
| Option<T> | generated tagged value or admitted `std::optional<T>` strategy |
| Result<T,E> | generated tagged value or admitted `std::variant` wrapper |
| Record | value class/struct with immutable portable surface |
| Tagged enum | explicit tag plus typed payload/value representation |
| Interface | generated owning type-erased value with a flat typed vtable |

Public portable APIs MUST NOT expose mutable container references, raw owning
pointers, reference lifetime obligations, or observable implementation
identity. Copy/move behavior is generated and verified from CoreIR ownership.

## 5. Declarations and control

- Portable constants use `constexpr`/`constinit` only when exact C++ rules
  permit; otherwise immutable initialization is explicit.
- Tagged matching uses exhaustive tag/variant visitation selected by a typed
  strategy.
- Explicit temporaries and blocks enforce CoreIR evaluation order where C++
  ordering would otherwise differ.
- Portable normal failure uses `Result`, not exceptions.
- All narrowing, overflow, shifts, division, float bits, and text conversion
  use checked typed operations with no undefined behavior.
- Allocation failure is outside ordinary portable `Result` unless the source
  program explicitly models allocation.

## 6. Interfaces, composition, and target heritage

Portable interfaces lower to owning type-erased values, not C++ abstract-base
inheritance. Each generated interface value contains an owning immutable erased
object and a flat typed operation table (or an equivalent verified
non-inheritance representation). The operation table includes typed
copy/move/destruction behavior. Dispatch does not expose the erased address or
RTTI. Multiple independent conformances and interface values nested in all
admitted portable types are supported.

Composition uses named value members and explicit delegation. Portable code
MUST NOT use base classes, virtual inheritance, mixins, CRTP reuse, or inherited
state.

`CppHeritage` may model the shared target-only one-edge adapter exception: one
generated final leaf may derive from one external framework base and delegate
to a composed component. Generated base classes, multiple/virtual inheritance,
chains, and reusable inherited implementation are rejected.

## 7. Symbols and includes

Closed catalogues include exact admitted types/functions from `cstdint`,
`cstddef`, `limits`, `bit`, `cmath`, `string`, `string_view`, `vector`, `array`,
`optional`, `variant`, `memory`, `utility`, `type_traits`, and generated runtime
symbols. Each known callable records namespace/owner, free/member/constructor/
operator invocation, template substitutions, cv/ref requirements, parameters,
result, `noexcept` policy, and lifetime/ownership effects.

The resolver derives system/local includes and forward declarations from typed
references, computes complete-type requirements, rejects cycles, and applies
deterministic namespace qualification for collisions. Lowerers never request
includes directly.

## 8. Runtime helpers

`CppRuntimeHelper` is a closed enum. Helpers expand to typed declarations and
definitions with explicit header/source placement and dependency closure.
Checked arithmetic, tagged values, Unicode/UTF-8, immutable bytes/lists, F64
bits, and type-erased interface support are structural AST. No runtime source
constant or feature-specific template exists.

## 9. File and package policy

Typed file roles select public header, private header, implementation, runtime,
test, and build files. The resolver owns include guards/`#pragma once` policy,
namespaces, declaration/definition splitting, internal linkage, initialization
order, and deterministic ordering. Public headers are self-contained and do
not leak private helpers.

## 10. Rendering

The C++ post-link checker certifies complete C++20 translation units as an
opaque `RenderReadyPackage<CppDialect>`. It validates declaration/declarator
shape, scopes, overload/call forms, templates used by the supported subset,
access, initialization, lifetime-bearing placement, and the optional one-edge
adapter heritage restriction after includes and helpers are resolved.

The total C++ renderer structurally covers files, includes, namespaces,
templates, declarations, definitions, types/declarators, initializers, blocks,
switches, statements, expressions, literals, comments, and tests. It owns every
C++ token and exhaustively matches the closed C++ AST. There is no executable
Handlebars template, token/source escape hatch, or renderer-side feature,
include, ownership, or overload decision.

A pinned formatter/compiler is an independent no-diff/acceptance oracle and
never a syntax or semantic repair.

## 11. Validation

The verifier checks types and value categories, cv/ref/lifetime requirements,
known/generated overload resolution, ownership and special members, exhaustive
tags, public immutability, interface operation-table completeness, heritage
policy, complete types, ODR-safe placement, includes, namespaces, sequencing,
precedence, and absence of opaque source.

## 12. Success evidence

- AST, ownership, declarator, catalogue, overload, and typed-call
  unit/compile-fail tests.
- Exact include/helper/placement matrices and collision/cycle fixtures.
- Type-erased flat-interface, multiple-conformance, copy/move/drop,
  first-class/nested interface-value, dispatch, delegation, and rejected
  inheritance-chain tests.
- Hermetic C++20 compile and native tests with strict warnings.
- GCC 14.2.0 AddressSanitizer and UndefinedBehaviorSanitizer runs.
- Separate public-header consumer and deliberate negative fixtures.
- Mutation/aliasing, ownership, Unicode, overflow, shifts, and F64-bit tests.
- Three-generation determinism and every historical/canonical conformance
  vector.

## 13. Migration exit

The C++ plugin passes only after raw source/runtime constants, abstract-base
portable contracts, and manual include registration are deleted and all
executable translation units flow through verified C++ AST, automatic linking,
and the render-ready certificate plus total structural renderer.
