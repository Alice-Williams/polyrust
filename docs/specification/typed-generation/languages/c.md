# C17 typed-generation specification

- Status: normative for M34A
- Target ID: `org.polyrust.c17`
- Language/toolchain: C17 through the hermetic Zig SDK, with GCC 14.2.0
  sanitizer verification

## 1. Scope and package

The plugin emits self-contained public headers, implementation/runtime C files,
native/conformance tests, negative compilation fixtures, and build metadata. A
separate translation unit MUST consume the public header without include-order
assumptions. The generated package has no undeclared third-party dependency.

## 2. Capability strategies

The exhaustive C registry maps every admitted CoreIR feature to a native or
explicitly emulated representation. It distinguishes checked scalars, exact
float bits, Unicode scalar/text ownership, monomorphized aggregates and tagged
values, immutable collection handles, and owning function-table interfaces.

Undefined behavior, implementation-sized integers, pointer identity, macros
with semantic behavior, and process termination MUST NOT approximate portable
semantics. Unsupported shapes produce a typed diagnostic before C AST exists.

## 3. C AST

The dialect owns:

- `CType`, qualifiers, pointers, arrays, function types, and full declarators;
- `CExpr` and `CPrecedence`;
- `CStmt`, `CBlock`, `CCase`, and initialization forms;
- `CDeclaration`, `CDefinition`, `CStorage`, and `CLinkage`;
- `CFile`, `CPackage`, `CTemplateId`, and typed header/source placement; and
- closed enums for operators, casts, call forms, includes, literals, tags, and
  preprocessor forms admitted solely for guards/platform assertions.

Function pointers and ownership operations are typed nodes. There is no raw
declaration/declarator escape and no executable C source string.

## 4. Type mapping

| CoreIR type | C17 representation |
| --- | --- |
| Unit | generated one-byte/empty-semantic value struct |
| Bool | `bool` from `<stdbool.h>` |
| I32 / I64 | `int32_t` / `int64_t` plus checked helpers |
| F64 | `double` with `memcpy`-based exact raw-bit helpers |
| Char | validated Unicode scalar stored as `uint32_t` |
| String | generated immutable owned UTF-8 value struct |
| Bytes | generated immutable owned byte value struct |
| List<T> | generated monomorphized immutable owned list struct |
| Option<T> | generated monomorphized tag/payload struct |
| Result<T,E> | generated monomorphized tag/payload struct |
| Record | generated value/owned struct with lifecycle functions as required |
| Tagged enum | generated explicit tag plus union/payload struct |
| Interface | generated owning context plus flat typed function table |

Every owning type has explicit initialization, clone/move policy, and drop
operations. Portable public APIs MUST NOT expose mutable backing storage,
unbounded pointers, sentinel-null options, or implicit borrowed lifetimes.

## 5. Declarations and control

- Constants use C constant expressions only when exact; other immutable values
  use deterministic initialization APIs with typed state.
- Generic CoreIR aggregates are monomorphized with stable collision-safe names.
- Tagged matches lower to exhaustive `switch` statements with guarded union
  access.
- Explicit temporaries sequence every receiver/argument because C operand and
  argument evaluation order cannot be assumed.
- Normal portable failure is returned as generated `Result`, never `errno`,
  `abort`, or an unchecked null.
- Arithmetic and shifts avoid signed overflow/invalid shift UB; float raw bits
  use `memcpy`; text and size conversions are checked.

## 6. Interfaces and composition

Each portable interface lowers to an owning immutable handle containing a
private context pointer and a pointer to a flat typed function table. The table
contains every interface method plus typed clone/move/drop operations required
by the ownership strategy. Constructors pair a concrete generated value with
the exact table; calls use typed nodes and never member-name strings.

The context address and table are not observable portable identity. Interface
values may be copied, moved, nested, stored in records/lists/options/results,
returned, and passed according to the explicit lifecycle contract. Multiple
independent conformances use distinct tables. Static dispatch remains a direct
typed function call.

Composition uses named struct fields and explicit forwarding functions. C has
no inheritance exception; prefix-layout inheritance, container-of downcasts,
and macro-generated polymorphism are forbidden.

## 7. Symbols and includes

Closed catalogues include exact admitted types/macros/functions from
`stdint.h`, `stdbool.h`, `stddef.h`, `limits.h`, `float.h`, `stdlib.h`,
`string.h`, `math.h`, and generated runtime declarations. A macro is catalogued
only as a typed constant/property, never as an opaque executable fragment.

Each callable records header, linkage, identifier, function-pointer/direct call
kind, parameter/result types, pointer constness, ownership transfer, aliasing,
failure behavior, and evaluation constraints. The resolver derives includes,
forward declarations, typedef/tag order, and header/source placement from typed
references; detects collisions/cycles; and never accepts manual include
requests from lowering.

## 8. Runtime helpers

`CRuntimeHelper` is a closed enum. Helpers expand to typed declarations and
definitions with explicit dependencies, ownership, linkage, and placement.
Checked arithmetic, tagged values, exact F64 bits, UTF-8/scalars, immutable
bytes/lists, allocation, and interface lifecycle/dispatch are structural AST.
There is no runtime source constant or feature-specific template.

Allocation helpers report typed failure where allocation is part of an
admitted operation. Cleanup paths are generated structurally and verified.

## 9. File and package policy

Typed file roles select public header, private/runtime header, implementation,
runtime implementation, tests, and build files. The resolver owns include
guards, C++ linkage compatibility guards where policy requires, declarations
versus definitions, static/internal linkage, initialization order, and
deterministic symbol/file ordering. Public headers are self-contained.

## 10. Rendering

`CTemplateId` covers files, guards, includes, linkage blocks, declarations,
definitions, types/declarators, initializers, blocks, switches, statements,
expressions, literals, comments, and tests. Strict embedded Handlebars receives
resolved C render views only. It performs no ownership, include, monomorphizing,
cleanup, or feature decisions.

A pinned formatter is a no-diff check/post-process and not a semantic repair.

## 11. Validation

The verifier checks complete declarator types, callable signatures, pointer
constness/ownership, initialization and exactly-once cleanup on every exit,
union-tag dominance, arithmetic UB avoidance, sequencing, monomorphization
uniqueness, interface table completeness, header/source linkage, complete
types, includes, precedence, exhaustive returns, and absence of opaque source.

## 12. Success evidence

- AST/declarator, ownership, catalogue, monomorphization, and typed-call
  unit/compile-fail tests.
- Exact include/helper/placement matrices and collision/cycle fixtures.
- Owning flat-interface, multiple-conformance, clone/move/drop,
  first-class/nested interface-value, dispatch, delegation, allocation-failure,
  and cleanup-path tests.
- Hermetic C17 compile and native tests with strict warnings.
- GCC 14.2.0 AddressSanitizer and UndefinedBehaviorSanitizer runs.
- Separate public-header consumer and deliberate negative fixtures.
- Mutation/aliasing, double-free/leak, Unicode, overflow, shifts, and F64 raw
  bit tests.
- Three-generation determinism and every historical/canonical conformance
  vector.

## 13. Migration exit

The C plugin passes only after raw source/runtime constants, macro-based
semantic shortcuts, and manual include registration are deleted and all
executable translation units flow through verified C AST, automatic linking,
and strict templates.
