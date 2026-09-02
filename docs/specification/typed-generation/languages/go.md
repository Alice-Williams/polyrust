# Go typed-generation specification

- Status: normative for M34A
- Target ID: `org.polyrust.go`
- Language/toolchain: Go 1.25.14

## 1. Scope and package

The plugin emits a self-contained Go module containing `go.mod`, generated
source files, an optional structural runtime file, native/conformance tests,
and compile-fail fixtures. Generated public APIs MUST be usable from a separate
consumer package. The module MUST have no undeclared third-party runtime
dependency.

## 2. Capability strategies

The exhaustive Go registry records one typed strategy for every CoreIR
feature. In particular it distinguishes native scalar and interface behavior
from emulated tagged values, exact floating-point bit operations, checked
integer operations, immutable collection boundaries, and Unicode validation.

Reflection, `unsafe`, panics, and implementation-defined integer width MUST NOT
be used to approximate portable behavior. An unsupported feature produces a
typed diagnostic before a Go package exists.

## 3. Go AST

The dialect owns:

- `GoType` and typed/generated type references;
- `GoExpr` and `GoPrecedence`;
- `GoStmt`, `GoBlock`, and `GoCase`;
- `GoDeclaration` and `GoReceiver`;
- `GoFile`, `GoPackage`, and `GoTemplateId`; and
- closed enums for operators, literals, visibility, declaration kinds, call
  forms, and import forms.

Pointer, slice, map, channel, variadic, selector, conversion, type-assertion,
and composite-literal syntax are explicit variants. No executable Go source
string enters this AST.

## 4. Type mapping

| CoreIR type | Go representation |
| --- | --- |
| Unit | generated zero-sized unit value/type |
| Bool | `bool` |
| I32 / I64 | `int32` / `int64` plus checked helpers |
| F64 | `float64` plus exact `math.Float64bits` behavior |
| Char | validated Unicode scalar represented by `rune` |
| String | UTF-8-valid `string` |
| Bytes | generated immutable value wrapper over copied `[]byte` |
| List<T> | generated immutable value wrapper over copied `[]T` |
| Option<T> | generated tagged generic value |
| Result<T,E> | generated tagged generic value |
| Record | value `struct` with unexported mutable representation where needed |
| Tagged enum | explicit tag plus typed payload representation |
| Interface | flat native Go interface over immutable value implementations |

`int`, mutable slices, maps, and pointers MUST NOT leak through portable public
types when doing so changes portable width, ownership, aliasing, or identity.

## 5. Declarations and control

- Portable constants use Go `const` only when Go can represent their exact
  value; otherwise they become immutable package initialization values.
- Records and tagged values use value semantics.
- Matches lower to exhaustive `switch` statements with typed tag/payload
  access; type switches are used only for a registered interface strategy.
- Explicit temporaries preserve CoreIR receiver and argument evaluation order.
- Portable `Result` flow is explicit and never implemented with panic/recover.
- Overflow, shifts, division, float bits, and scalar/string conversions call
  typed registered helpers where native Go differs from CoreIR.

## 6. Interfaces and composition

Portable interfaces lower to flat Go `interface` declarations. Generated
interfaces MUST NOT embed another interface. Implementations expose exactly the
required methods and carry no inherited implementation. Compile-time
conformance assertions are emitted as typed declarations.

First-class interface values use ordinary Go interface dispatch over immutable
value implementations. Generated portable APIs MUST NOT expose pointer identity
or typed-nil ambiguity. Multiple independent interface conformances are
allowed. Composition uses explicitly named struct fields and explicit
delegation; anonymous field embedding and promoted methods are forbidden for
portable behavior.

Go has no class-inheritance escape hatch. Target-only framework adaptation
therefore also uses composition and delegation.

## 7. Symbols and imports

Closed catalogues include the exact admitted members of `math`, `unicode/utf8`,
`encoding/binary`, `bytes`, `errors`, and any other standard package selected by
a capability strategy. Each known callable records package, member, invocation
kind, parameter/result patterns, generic instantiation rules, and failure
behavior.

The resolver derives imports solely from typed references, removes unused
imports, groups standard/generated imports deterministically, and selects
stable aliases for collisions. A language lowerer MUST NOT call an import
requirement helper.

## 8. Runtime helpers

`GoRuntimeHelper` is a closed enum. Each helper expands to Go AST declarations
and typed dependencies. Tagged option/result, checked integer, F64 bit, Unicode,
byte, immutable list, and interface-support helpers are structural declarations
rather than source constants.

No `runtime.go` body constant or feature-specific rendering template exists.

## 9. File and package policy

The resolver assigns declarations to generated, runtime, and test files using
typed file roles. All non-test files share one declared package. File names,
declaration ordering, build constraints, visibility, and initialization order
are deterministic. Cyclic initialization or import relationships are rejected
before rendering.

## 10. Rendering

`GoTemplateId` covers package clauses, imports, declarations, types, blocks,
statements, expressions, literals, comments, and tests. Strict embedded
Handlebars templates receive resolved Go render views only. Templates contain
grammar skeletons, not runtime helpers or portable feature implementations.

`gofmt` is a pinned no-diff check/post-process and MUST NOT repair semantic or
structural omissions.

## 11. Validation

The Go verifier checks expression and declaration types, known/generated call
signatures, receiver legality, flat-interface conformance, absence of embedded
portable fields/interfaces, immutable public boundaries, exhaustive tagged
control, import use, initialization cycles, precedence, and absence of opaque
source.

## 12. Success evidence

- AST, symbol-catalogue, and typed known-call unit/compile-fail tests.
- Exact import/helper dependency matrices and collision fixtures.
- Flat-interface, multiple-conformance, first-class/nested interface-value,
  dynamic-dispatch, and explicit-delegation tests.
- `gofmt` no-diff, `go vet`, and `go test ./...` under the pinned toolchain.
- External consumer-package tests and deliberate compile-fail fixtures.
- Mutation/aliasing, typed-nil, Unicode, arithmetic, and F64-bit boundaries.
- Three-generation determinism and every historical/canonical conformance
  vector.

## 13. Migration exit

The Go plugin passes only after raw Go/runtime constants and manual import
registration are deleted, public mutable slice aliases are eliminated, and all
executable package files flow through verified Go AST and strict templates.
