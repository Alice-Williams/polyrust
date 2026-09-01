# Target-language IR compliance ledger

Status: M30 migration baseline

This ledger audits the eight supported outputs against the normative
[compositional target-language IR contract](language-ir-architecture.md).
`Pass` requires executable evidence. `Partial` identifies an existing useful
boundary that does not satisfy the complete invariant. `Fail` is migration work,
not an accepted exception.

## Baseline audit

| Surface | Structured renderer | Mapping-local fragments | Helper graph/minimal runtime | Source-role closure | No repair scan | Baseline result |
| --- | --- | --- | --- | --- | --- | --- |
| Shared codegen | Pass: renderer sees `ImportSet` only | Pass: associative `LanguageFragment` composition closes immutable units | Pass: deterministic closure rejects invalid, duplicate, missing, and cyclic helper graphs | Partial: source roles can still use raw `Text` | Pass: closed units expose no document/dependency repair API | **Fail** |
| Rust | Pass: `RustImport` renderer | Fail: declarations build one `String` | Fail: complete `RUNTIME` is copied as text | Fail: runtime uses `LanguageFile::text` | Partial: source imports are attached at file-unit scope | **Fail** |
| TypeScript | Pass: validated default/named/type-only/export IR; renderer alone spells directives | Pass: paired `EcmaCode` composes types, declarations, and tests | Pass: exact intrinsic-selected paired helper closure | Pass: every generated TypeScript source role uses a source file | Pass: mapping-local fragments replace file-wide attachment | **Pass** |
| JavaScript | Pass: the same validated `EcmaImport` IR with type-only erasure | Pass: erased syntax comes from the same `EcmaCode` traversal | Pass: compiler-derived runtime has the same helper IDs and roots | Pass: every generated JavaScript source role uses a source file | Pass: no parallel declaration or dependency scan remains | **Pass** |
| Python | Pass: validated future/module/from IR; renderer alone spells imports | Pass: `PythonCode` composes types, values, declarations, and portable tests | Pass: exact common and optional F64 helper closure | Pass: every generated Python source role uses a source file | Pass: dependency repair walks are deleted | **Pass** |
| Go | Pass: validated path IR; renderer alone spells `import` | Pass: `GoCode` composes types, values, declarations, and portable tests | Pass: exact common/integer/F64/bytes/text helper roots | Pass: every generated Go source role uses a source file | Pass: F64 values own `math`; the nested-value repair scan is deleted | **Pass** |
| Java | Pass: validated kind/name IR; renderer alone spells `import` | Pass: nested type and declaration `JavaCode` fragments own imports | Pass: checked-program roots resolve ordered common, numeric, and UTF-8 helper closures | Pass: every generated Java source role uses a source file | Pass: source and runtime dependencies originate in the fragment that emits dependent syntax | **Pass** |
| C++ | Pass: structured system/local includes | Fail: declarations build file-sized strings | Fail: runtime has a fixed 17-header inventory | Pass: generated C++ code uses source files | Fail: header declarations/capabilities are rescanned for includes | **Fail** |
| C | Pass: renderer owns stripped system/local includes | Fail: generator returns file-sized strings | Fail: runtime header/source use fixed inventories | Pass: generated C code uses source files | Partial: dependencies are attached only at file-unit scope | **Fail** |

The audit deliberately does not infer compliance from successful compilation.
The current full and release gates prove functional generated output; they do
not prove dependency completeness or minimality.

## Evidence locations

- Shared mutable unit and source-role bypass APIs:
  `crates/codegen/src/language.rs`.
- Rust monolithic runtime/source assembly:
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
- C++ fixed runtime headers and header feature scans:
  `crates/backend-cpp/src/lib.rs`.
- C fixed runtime/file-level requirements:
  `crates/backend-c/src/lib.rs`.

## Closure rule

A row moves to `Pass` only in the commit that adds all contract tests for that
surface. Each migration records exact minimal/feature matrix coverage here.
M30 is complete only when every row passes, the policy test prevents regression,
all generated packages remain functionally equivalent, and hosted CI is green.

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
