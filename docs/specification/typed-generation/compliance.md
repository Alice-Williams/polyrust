# Typed-generation compliance ledger

- Status: normative baseline for M34A
- Baseline date: 2026-09-02
- Supersedes for new work: `docs/language-ir-compliance.md`

This ledger measures the stronger ADR-0004 contract. The M30 ledger remains
historical evidence that the existing fragment paths carry dependency metadata;
it is not evidence of typed executable syntax.

## Baseline audit

| Surface | CoreIR input | Typed executable AST | Typed symbols and derived dependencies | Structural runtime | Resolved-only strict renderer | Baseline |
| --- | --- | --- | --- | --- | --- | --- |
| Shared codegen | Missing | Missing | Partial fragment metadata | Partial helper closure over documents | Missing | **Fail** |
| Rust | Missing | Missing: `RustCode`/raw documents | Manually attached fragment metadata | Raw runtime source | Documents are directly rendered | **Fail** |
| TypeScript | Missing | Missing: paired `EcmaCode` | Manually attached fragment metadata | Raw runtime source | Paired source path | **Fail** |
| JavaScript | Missing | Independently paired source exists | Shares manual ECMA metadata | Checked-in/runtime paired text | Not solely compiler-derived | **Fail** |
| Python | Missing | Missing: generated source fragments | Manually attached fragment metadata | Raw runtime source | Documents are directly rendered | **Fail** |
| Go | Missing | Missing: generated source fragments | Manually attached fragment metadata | Raw runtime source | Documents are directly rendered | **Fail** |
| Java | Missing | Missing: `JavaCode`/raw documents | `require_java` accepts qualified-name strings | Included/parceled runtime source | Documents are directly rendered | **Fail** |
| C++20 | Missing | Missing: `CppCode`/raw documents | Manually attached fragment metadata | Included/sectioned runtime source | Documents are directly rendered | **Fail** |
| C17 | Missing | Missing: `CCode`/raw documents | Manually attached fragment metadata | Included/sectioned runtime source | Documents are directly rendered | **Fail** |

## Evidence locations

- Shared fragment/document escape:
  `crates/codegen/src/language.rs` and `crates/codegen/src/document.rs`.
- Rust: `crates/backend-rust/src/v0.rs`.
- TypeScript/JavaScript: `crates/backend-typescript/src/lib.rs` and
  `crates/backend-typescript/src/runtime.ts`.
- Python: `crates/backend-python/src/lib.rs` and
  `crates/backend-python/src/runtime.py`.
- Go: `crates/backend-go/src/v0.rs` and
  `crates/backend-go/src/runtime.go`.
- Java: `crates/backend-java/src/lib.rs` and
  `crates/backend-java/src/Runtime.java`.
- C++20: `crates/backend-cpp/src/lib.rs` and
  `crates/backend-cpp/src/runtime.hpp`.
- C17: `crates/backend-c/src/lib.rs`,
  `crates/backend-c/src/generator.rs`, `crates/backend-c/src/runtime.h`, and
  `crates/backend-c/src/runtime.c`.

These locations are migration inputs, not allowlisted final architecture.

## Pass rule

A row moves to **Pass** only in the same checkpoint which:

1. satisfies all nine shared layer specifications;
2. satisfies every section of the corresponding language specification;
3. deletes the language's legacy raw/paired executable path;
4. adds permanent positive, negative, compile-fail, and source-policy proof;
5. passes its native formatter, linter/static analysis, compiler, and tests;
6. passes three-generation determinism and canonical semantic conformance; and
7. replays every historical real-world port for that language.

JavaScript passes only with TypeScript and only after clean pinned compiler
derivation proves every executable JavaScript byte.

## Release rule

M34A is complete only when the shared row and all eight language rows are
**Pass**, the old M30 ledger is clearly historical, and uncached repository,
release, and hosted CI gates are green.
