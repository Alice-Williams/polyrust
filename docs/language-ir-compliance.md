# Target-language IR compliance ledger

Status: M30 complete — all shared and target rows pass

This ledger audits the eight supported outputs against the normative
[compositional target-language IR contract](language-ir-architecture.md).
`Pass` requires executable evidence. `Partial` identifies an existing useful
boundary that does not satisfy the complete invariant. `Fail` is migration work,
not an accepted exception.

## Baseline audit

| Surface | Structured renderer | Mapping-local fragments | Helper graph/minimal runtime | Source-role closure | No repair scan | Baseline result |
| --- | --- | --- | --- | --- | --- | --- |
| Shared codegen | Pass: renderer sees `ImportSet` only | Pass: associative `LanguageFragment` composition closes immutable units | Pass: deterministic closure rejects invalid, duplicate, missing, and cyclic helper graphs | Pass: disjoint role types and private variants make raw source files unconstructable | Pass: closed units expose no document/dependency repair API | **Pass** |
| Rust | Pass: validated module/use IR; renderer alone spells directives | Pass: `RustCode` composes every syntax mapping and portable test | Pass: exact common/replacement/truncation/shift helper closure | Pass: every generated Rust source role uses a source file | Pass: intrinsic mappings directly own helper roots | **Pass** |
| TypeScript | Pass: validated default/named/type-only/export IR; renderer alone spells directives | Pass: paired `EcmaCode` composes types, declarations, and tests | Pass: exact intrinsic-selected paired helper closure | Pass: every generated TypeScript source role uses a source file | Pass: mapping-local fragments replace file-wide attachment | **Pass** |
| JavaScript | Pass: the same validated `EcmaImport` IR with type-only erasure | Pass: erased syntax comes from the same `EcmaCode` traversal | Pass: compiler-derived runtime has the same helper IDs and roots | Pass: every generated JavaScript source role uses a source file | Pass: no parallel declaration or dependency scan remains | **Pass** |
| Python | Pass: validated future/module/from IR; renderer alone spells imports | Pass: `PythonCode` composes types, values, declarations, and portable tests | Pass: exact common and optional F64 helper closure | Pass: every generated Python source role uses a source file | Pass: dependency repair walks are deleted | **Pass** |
| Go | Pass: validated path IR; renderer alone spells `import` | Pass: `GoCode` composes types, values, declarations, and portable tests | Pass: exact common/integer/F64/bytes/text helper roots | Pass: every generated Go source role uses a source file | Pass: F64 values own `math`; the nested-value repair scan is deleted | **Pass** |
| Java | Pass: validated kind/name IR; renderer alone spells `import` | Pass: nested type and declaration `JavaCode` fragments own imports | Pass: checked-program roots resolve ordered common, numeric, and UTF-8 helper closures | Pass: every generated Java source role uses a source file | Pass: source and runtime dependencies originate in the fragment that emits dependent syntax | **Pass** |
| C++ | Pass: validated system/local include IR; renderer alone spells directives | Pass: `CppCode` composes types, declarations, definitions, conversions, and tests | Pass: source-owned roots resolve marked model/JSON/engine runtime nodes | Pass: every generated C++ source role uses a source file | Pass: declaration/capability repair scans and fixed runtime inventory are deleted | **Pass** |
| C | Pass: validated system/local include IR; renderer alone spells directives | Pass: `CCode` composes ABI types, declarations, definitions, ownership, expressions, values, and tests | Pass: exact core and optional semantic runtime-helper closure | Pass: every generated C source role uses a source file | Pass: syntax-producing mappings directly own includes and helper roots | **Pass** |

The audit deliberately does not infer compliance from successful compilation.
The current full and release gates prove functional generated output; they do
not prove dependency completeness or minimality.

`Pass` in this ledger is deliberately narrower than complete PolyIR feature
coverage. It means that every construct the backend accepts obeys the
dependency-ownership contract and that unsupported constructs fail with a
diagnostic before rendering. The separate target-expansion milestones track
semantic coverage; in particular, M22/M22B remains in progress without
weakening the C row here.

## Evidence locations

- Shared fragments, helper closure, and role-safe file APIs:
  `crates/codegen/src/language.rs`.
- Rust fragments and helper-closure implementation:
  `crates/backend-rust/src/v0.rs`.
- TypeScript/JavaScript paired fragments and helper closure:
  `crates/backend-typescript/src/lib.rs`.
- Python fragment and helper-closure implementation:
  `crates/backend-python/src/lib.rs`.
- Go fragment and helper-closure implementation:
  `crates/backend-go/src/v0.rs` and `crates/backend-go/src/runtime.go`.
- Java fragment and helper-closure implementation:
  `crates/backend-java/src/lib.rs` and
  `crates/backend-java/src/Runtime.java`.
- C++ fragments and marked runtime-helper closure:
  `crates/backend-cpp/src/lib.rs` and
  `crates/backend-cpp/src/runtime.hpp`.
- C fragments and marked runtime-helper closure:
  `crates/backend-c/src/lib.rs`, `crates/backend-c/src/generator.rs`,
  `crates/backend-c/src/runtime.h`, and `crates/backend-c/src/runtime.c`.

## Closure rule

A row moves to `Pass` only in the commit that adds all contract tests for that
surface. Each migration records exact minimal/feature matrix coverage here.
M30 is complete only when every row passes, the policy test prevents regression,
all generated packages remain functionally equivalent, and hosted CI is green.

That closure condition is satisfied by hosted workflow
[33577166696](https://github.com/Alice-Williams/polyrust/actions/runs/33577166696)
at `64cec7defbad6b61c56511fc5a986fdb1b08ecf2`.

## Migration evidence

### Java source mapping (M30-02)

- Structured Java imports validate qualified names and distinguish normal type
  imports from static-member imports; rendered directives and wildcards are
  rejected as data.
- Type, parameter, contract, record, enum, constant, and function mappings
  compose `JavaCode` fragments. Nested list types propagate `java.util.List`
  without a second type walk.
- Record/enum fragments own collection/map dependencies and `Generated.java`
  folds declaration fragments; the old declaration pre-scan is deleted.
- Minimal/rich output and declaration-isolation matrices prove exact presence
  and absence. Java 21 native, conformance, and public-consumer tests pass with
  Buildifier, Rustfmt, and Clippy.

### Shared helper graph and Java runtime (M30-03)

- Shared `RuntimeHelperGraph` validates IDs and resolves deterministic,
  deduplicated transitive closures. Missing roots/dependencies and cycles are
  diagnostics rather than partial output.
- The Java runtime template is an ordered registry of common, numeric, and
  UTF-8 fragments. Helper marker metadata is removed before rendering; each
  node owns its structured import requirements.
- Numeric roots are selected from checked capabilities. UTF-8 roots use a
  semantic expression/constant/block visitor, avoiding the incorrect inference
  that every bytes-using program needs charset decoding.
- Empty and registration fixtures exclude `BigInteger` and NIO. Numeric-only
  and UTF-8-only checked programs prove exact mutually exclusive closures.
- Java 21 native, conformance, and public-consumer targets pass together with
  Java unit tests, Buildifier, Rustfmt, and Clippy.

### Go fragments and runtime (M30-04A)

- `GoCode` composition carries structured import requirements through nested
  types, values, declarations, result conversion, and portable tests.
- Go import paths are validated semantic data. Rendered directives, traversal,
  rooted paths, invalid separators, and invalid characters are rejected.
- Runtime closure independently selects checked integers, F64, bytes replace,
  scalar length, replace-many, truncate-UTF-8, and UTF-8 decoding. The fixed
  import loop and unused `encoding/binary` sentinel are gone.
- Empty and exact one-feature checked programs prove positive and negative
  import/helper matrices. Nested F64 values prove `math` propagation without a
  second value walk.
- Go unit/native gates (`gofmt`, `go vet`, `go test`), Rustfmt, Clippy, and all
  130 tests in `//examples/real-world/...` pass.

### Python fragments and runtime (M30-04B)

- `PythonCode` fragments carry validated future, module, and from-import
  requirements through nested types, declarations, implementations, tests,
  and callable bodies.
- Direct module imports and relative from-imports have distinct validation;
  rendered directives cannot enter dependency data.
- The declaration/type repair passes are deleted. A nested
  `Result<Option<I64>, String>` matrix proves dependency propagation from the
  syntax-producing type mapping.
- Runtime common and F64 nodes form an ordered helper graph with exact
  `dataclasses`, `types`, `typing`, `math`, and `struct` ownership. Empty
  and F64 fixtures prove positive and negative closure and marker-free output.
- Backend unit tests, generated Python lint/type/native/conformance/public
  checks, Rustfmt, Clippy, and all 130 real-world tests pass.

### TypeScript and derived JavaScript fragments (M30-04C)

- `EcmaCode` owns paired TypeScript and erased JavaScript syntax plus
  validated structured dependencies. Types, parameters, declarations, and
  portable tests compose it in one traversal; the old parallel JavaScript
  emitter and naked-string type mapping are deleted.
- Type-only imports propagate through nested
  `Result<Option<I64>, String>` and disappear with erased syntax. Default,
  named, type-only, and export-all import data reject invalid module and symbol
  forms, and only the renderer spells directives.
- TypeScript runtime markers are retained by compiler-derived JavaScript.
  Layout parity is mandatory, and exact roots select optional declaration and
  dispatch-case pairs for replacement, truncation, trimming, concatenation,
  and UTF-8 operations.
- Minimal and one-feature matrices prove exact helper absence/presence in both
  dialects. Runtime, index, test, conformance, negative-test, and Node-shim
  source roles all use closed source files.
- Three-generation determinism, Prettier, strict `tsc`, Node TypeScript and
  standalone JavaScript tests, derivation parity, and all 130 real-world tests
  pass.

### Rust fragments and runtime (M30-04D)

- `RustCode` composes syntax, validated imports, and helper roots through
  nested types, values, constants, expressions, blocks, calls, patterns,
  declarations, documentation, and portable tests. Only identifier/package
  allocation and test-only rendering helpers return strings.
- `RustImport` privately stores validated module or use-path data with
  test-only/public policy. Invalid identifiers, malformed paths, and rendered
  directives are rejected; only `RustRenderer` spells `mod` and `use`.
- Runtime replacement, byte replacement, UTF-8 truncation, and checked-shift
  helpers are stable nodes. Intrinsic mappings own their exact roots, and
  deterministic closure retains only the permanent error/result/list-get core
  plus selected helpers.
- Minimal and one-feature helper matrices, nested list/option/result type
  composition, mapping-local root assertions, and marker-free output are
  executable tests. Runtime, source, test, and conformance roles are closed
  source files.
- Three-generation determinism, generated-crate Rustfmt/Clippy,
  debug/release/native and negative-compilation gates, and all 130 real-world
  tests pass.

### C++ fragments and runtime (M30-04E)

- `CppImport` privately stores validated system/local include paths. Empty,
  rooted, traversal, malformed, non-header local, and rendered-directive forms
  are rejected; only `CppRenderer` spells `#include`.
- `CppCode` composes types, parameters, forward declarations, type bodies,
  callable declarations, definitions, value bridges, conversion bridges,
  tests, and runtime roots. The old declaration/capability header scans and
  file-sized header/source assembly are deleted.
- The runtime template is divided into ordered model, JSON, and execution
  nodes. Each node owns its exact headers, while the generated runtime
  bootstrap owns `runtime.full`; deterministic closure removes marker metadata
  and merges requirements.
- Empty/scalar/string/numeric/bytes/nested container type matrices and exact
  per-runtime-node/full-runtime header matrices are executable tests. Runtime,
  source, test, and conformance roles are closed source files.
- Three-generation determinism, generated C++20 warnings-as-errors,
  conformance, public-consumer, style, ASan/UBSan, Rustfmt, Clippy, and all 130
  real-world tests pass.

### C fragments and runtime (M30-04F)

- `CImport` privately stores validated system/local include paths. Empty,
  rooted, traversal, malformed, non-header local, and rendered-directive forms
  are rejected; only `CRenderer` spells `#include`.
- `CCode` carries syntax, structured includes, and runtime roots through ABI
  types, declarations, definitions, ownership helpers, expressions, literals,
  portable-test values, and result assertions. Dependency-free name allocation
  and indentation remain spelling helpers rather than dependency boundaries.
- Marked runtime header/source sections form deterministic core, F64, string,
  bytes, replacement, truncation, and trimming helper nodes. Programs select
  roots from the fragments that emit calls; the fixed all-features runtime root
  and fixed include inventories are deleted.
- Empty and primitive/composite ABI matrices prove exact includes. Core and
  one-feature runtime matrices prove positive and negative helper/import
  closure, marker removal, and minimal-program exclusion of optional nodes.
- Three-generation determinism, header self-containment, C17
  warnings-as-errors, ABI/ownership, native, conformance, public-consumer,
  style, ASan/UBSan, Rustfmt, Clippy, and all 130 real-world tests pass.

### Shared enforcement and extension proof (M30-05)

- `SourceFileRole` is accepted only by `LanguageSourceFile`; `TextFileRole` is
  accepted only by raw text files; byte files are assets. Private
  `LanguageFile` variants prevent direct construction from bypassing those
  roles. Shared unit tests and Bazel Rust doctests prove both positive and
  compile-fail boundaries.
- `//tools/source-policy:source_policy_test` scans every backend production Rust
  string and checked-in target template. Dependency directives are legal only
  inside `render_imports`; handwritten C, C++, and Java consumers use a
  path-exact allowlist. A separate fault-injection target proves Rust/template
  injections fail, a same-named free function receives no renderer permission,
  and copied fixture paths receive no exception. The external-backend proof is
  scanned with the built-in targets.
- The historical Go registration fixture now supplies its conditional test
  dependency to an import renderer instead of embedding a directive in the
  generated body.
- `examples/external-backend` implements the same public `LanguagePlugin`
  boundary with a structured import key, mapping-local fragment, helper graph,
  closed source file, syntax-only renderer, registry, preflight, and repeated
  generation contract proof.
- The complete uncached repository test run passes 201/201 tests. The dedicated
  uncached release gate passes 178/178 tests, including Buildifier, Rustfmt,
  Clippy, all target-native checks, public consumers, differential tests,
  C/C++ sanitizers, documentation checks, and both source-policy tests.
