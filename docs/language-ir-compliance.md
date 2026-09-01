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
| TypeScript | Pass: `EcmaImport` renderer | Fail: declarations build one body string | Fail: complete runtime is copied as text | Fail: runtime and node shim source bypass source IR | Partial: imports are attached to whole file bodies | **Fail** |
| JavaScript | Pass: derived `EcmaImport` renderer | Fail: declarations build one body string | Fail: complete derived runtime is copied as text | Fail: runtime source bypasses source IR | Partial: imports are attached to whole file bodies | **Fail** |
| Python | Pass: structured future/module/from renderer | Fail: declaration text and requirements mutate one unit | Fail: monolithic runtime has a fixed inventory | Pass: generated Python code uses source files | Fail: type/declaration walks separately attach imports | **Fail** |
| Go | Pass: `GoImport` renderer | Fail: declarations build one body string | Fail: `go_runtime_file` has a fixed import loop | Pass: generated Go code uses source files | Fail: test values are rescanned to decide `math` | **Fail** |
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
- TypeScript/JavaScript raw runtime and shim source files:
  `crates/backend-typescript/src/lib.rs`.
- Python runtime inventory and declaration import walks:
  `crates/backend-python/src/lib.rs`.
- Go runtime loop and portable-test F64 rescan:
  `crates/backend-go/src/v0.rs`.
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
