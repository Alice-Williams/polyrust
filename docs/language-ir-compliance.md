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
| Shared codegen | Pass: renderer sees `ImportSet` only | Fail: mutable file-sized `LanguageUnit` | Fail: no helper graph in language IR | Partial: source roles can still use raw `Text` | Fail: `set_document` and late requirement mutation permit repair | **Fail** |
| Rust | Pass: `RustImport` renderer | Fail: declarations build one `String` | Fail: complete `RUNTIME` is copied as text | Fail: runtime uses `LanguageFile::text` | Partial: source imports are attached at file-unit scope | **Fail** |
| TypeScript | Pass: `EcmaImport` renderer | Fail: declarations build one body string | Fail: complete runtime is copied as text | Fail: runtime and node shim source bypass source IR | Partial: imports are attached to whole file bodies | **Fail** |
| JavaScript | Pass: derived `EcmaImport` renderer | Fail: declarations build one body string | Fail: complete derived runtime is copied as text | Fail: runtime source bypasses source IR | Partial: imports are attached to whole file bodies | **Fail** |
| Python | Pass: structured future/module/from renderer | Fail: declaration text and requirements mutate one unit | Fail: monolithic runtime has a fixed inventory | Pass: generated Python code uses source files | Fail: type/declaration walks separately attach imports | **Fail** |
| Go | Pass: `GoImport` renderer | Fail: declarations build one body string | Fail: `go_runtime_file` has a fixed import loop | Pass: generated Go code uses source files | Fail: test values are rescanned to decide `math` | **Fail** |
| Java | Pass: renderer alone spells `import` | Fail: declarations build one body string | Fail: `java_runtime_file` has a fixed import loop | Pass: generated Java code uses source files | Fail: `Generated.java` prescans records/enums for imports | **Fail** |
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
- Java runtime loop and record/enum pre-scan:
  `crates/backend-java/src/lib.rs`.
- C++ fixed runtime headers and header feature scans:
  `crates/backend-cpp/src/lib.rs`.
- C fixed runtime/file-level requirements:
  `crates/backend-c/src/lib.rs`.

## Closure rule

A row moves to `Pass` only in the commit that adds all contract tests for that
surface. Each migration records exact minimal/feature matrix coverage here.
M30 is complete only when every row passes, the policy test prevents regression,
all generated packages remain functionally equivalent, and hosted CI is green.
