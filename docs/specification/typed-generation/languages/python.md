# Python typed-generation specification

- Status: normative for M34A
- Target ID: `org.polyrust.python`
- Language/toolchain: Python 3.13.5, Ruff 0.16.5, mypy 2.3.1, pytest 9.1.1

## 1. Scope and package

The plugin emits:

- `pyproject.toml`;
- `src/generated_polyrust/__init__.py`;
- `src/generated_polyrust/runtime.py` when required;
- generated native/conformance tests; and
- strict negative type fixtures.

The package has no undeclared third-party runtime dependency.

## 2. Capability strategies

The exhaustive Python registry distinguishes native immutable semantics from
fixed-width, exact-bit, Unicode, and tagged-value emulation.

Every CoreIR feature has an explicit typed strategy or unsupported reason.
Python dynamic behavior is never used to bypass portable static types.

## 3. Python AST

The dialect owns:

- `PythonType`;
- `PythonExpr` and `PythonPrecedence`;
- `PythonStmt`;
- `PythonPattern` where admitted;
- `PythonDeclaration`;
- `PythonDecorator` and `PythonVisibility`;
- `PythonFile`; and
- closed Python grammar and formatting categories.

Decorators, operators, import forms, parameter kinds, and declaration kinds
are enums. No executable source string enters the AST.

## 4. Type mapping

| CoreIR type | Python representation |
| --- | --- |
| Unit | generated frozen unit value/type |
| Bool | `bool` |
| I32 / I64 | `int` guarded by exact-width helpers |
| F64 | `float` with `struct` exact-bit helpers |
| Char | validated one-scalar `str` |
| String | surrogate-free `str` |
| Bytes | immutable `bytes` |
| List<T> | immutable `tuple[T, ...]` |
| Option<T> | generated tagged generic, never `None` |
| Result<T,E> | generated tagged generic |
| Record | frozen, slotted dataclass |
| Tagged enum | closed frozen tagged variants |
| Interface | flat `typing.Protocol` view over immutable implementations |

Public generated source contains no unbounded `Any` escape.

## 5. Declarations and control

- Constants are typed module bindings with immutable representations.
- Records are frozen/slotted dataclasses.
- Result/option/enum tags are explicit.
- CoreIR matches lower to explicit tag/type control with exhaustive verifier
  coverage.
- Fixed-width operations use registered helpers.
- Exceptions do not implement normal portable `Result` flow.
- Explicit temporaries preserve receiver/argument evaluation order.

## 6. Interfaces and composition

Portable interfaces lower to flat Protocol declarations and exact typed
implementations. Inheriting from `Protocol` is a Python conformance marker, not
portable implementation inheritance. Generated interfaces do not inherit from
other interfaces and implementations do not inherit method bodies.

First-class interface values are ordinary immutable implementation objects
typed as the Protocol. Mypy fixtures prove method signatures and nested
interface positions. Composition uses dataclass fields and explicit delegation.

Mixins, multiple implementation inheritance, `super` delegation, metaclass
dispatch, monkey patching, and implicit member forwarding are forbidden.

## 7. Symbols and imports

Closed catalogues cover exact used members of:

- `dataclasses`;
- `typing`;
- `math`;
- `struct`;
- `types`; and
- Python builtins.

Each known callable owns module, name, invocation kind, parameter/result
patterns, and failure behavior. The resolver distinguishes module imports,
from-imports, relative imports, builtins, aliases, type-only dependencies, and
collisions.

No lowering function calls an import-requirement helper.

## 8. Runtime helpers

`PythonRuntimeHelper` is a closed enum. Helpers are Python AST declarations.
The common/tagged runtime and optional integer/F64/text/bytes helpers have exact
typed dependency closure.

No `runtime.py` body constant or feature-specific template exists.

## 9. File and package policy

Typed file roles select package metadata, package initializers, generated
modules, runtime modules, native/conformance tests, and negative fixtures. The
resolver owns relative module paths, exported names, type-only dependencies,
deterministic file names, and import-cycle diagnostics. Package initializers
re-export only symbols selected structurally by the package policy.

## 10. Rendering

The Python post-link checker certifies complete modules as an opaque
`RenderReadyPackage<PythonDialect>`. It validates indentation-bearing suite
shape, scope directives and bindings, decorator/Protocol placement, parameter
ordering, assignment targets, loop/exception context, match patterns, returns,
imports, and annotation forms for the pinned Python grammar.

The total Python renderer structurally covers files, imports, decorators,
classes, Protocols, functions, methods, annotations, blocks, statements,
expressions, patterns, literals, and tests. It owns Python precedence,
indentation, identifiers, literal escaping, and annotation spelling through
exhaustive AST matches. There is no executable Handlebars template or
token/source escape hatch.

The pinned Python parser/compiler and Ruff are independent acceptance/no-diff
oracles, not syntax repair or certificate constructors.

## 11. Validation

The verifier checks:

- expression/annotation types;
- known/generated callable signatures;
- Protocol method conformance;
- no inherited implementation/mixins;
- frozen collection/record exposure;
- exhaustive tags;
- import namespaces and relative levels;
- precedence; and
- no opaque source.

## 12. Success evidence

- AST/typed known-call compile-fail and verifier tests.
- Exact import and helper matrices.
- Flat Protocol, multiple conformance, first-class/nested interface values,
  dynamic dispatch, and explicit delegation.
- `python3 -m compileall`.
- `ruff format --check` and `ruff check`.
- `mypy --strict` including deliberate negative fixtures.
- `pytest` for portable/native/conformance tests.
- Three-generation determinism.
- Mutation/aliasing and surrogate boundary tests.
- Every historical port and canonical conformance vector.

## 13. Migration exit

The Python plugin passes only after `PythonCode`/raw source/runtime constants
are deleted and all executable package files flow through typed Python AST and
the render-ready certificate and total structural renderer.
