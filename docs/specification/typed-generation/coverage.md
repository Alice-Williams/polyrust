# Normative specification coverage

- Status: normative cross-audit for M34A
- Audited: 2026-09-02

This matrix names the specification which owns each split language concern. A
blank or inherited-by-convention cell is not permitted. JavaScript is the only
intentional derived-language exception.

| Concern | Rust | TypeScript | JavaScript | Python | Go | Java | C++20 | C17 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Target identity/toolchain | rust.md §1 | typescript.md §1 | javascript.md §1-2 | python.md §1 | go.md §1 | java.md §1 | cpp.md §1 | c.md §1 |
| Capability strategy | §2 | §2 | §3 | §2 | §2 | §2 | §2 | §2 |
| Typed executable AST | §3 | §3 | prohibited by §1 | §3 | §3 | §3 | §3 | §3 |
| Portable type mapping | §4 | §4 | erasure in §4 | §4 | §4 | §4 | §4 | §4 |
| Declarations/control | §5 | §5 | compiler-derived by §1-2 | §5 | §5 | §5 | §5 | §5 |
| Interfaces/composition | §6 | §6 | §4 | §6 | §6 | §6 | §6 | §6 |
| Symbols/dependencies | §7 | §7 | §5 | §7 | §7 | §7 | §7 | §7 |
| Structural runtime | §8 | §8 | §6 | §8 | §8 | §8 | §8 | §8 |
| Files/package | §9 | §9 | §2 and §7 | §9 | §9 | §9 | §9 | §9 |
| Rendering | §10 | §10 | executable renderer prohibited by §7 | §10 | §10 | §10 | §10 | §10 |
| AST/package validation | §11 | §11 | derivation validation in §8 | §11 | §11 | §11 | §11 | §11 |
| Required evidence | §12 | §12 | §9 | §12 | §12 | §12 | §12 | §12 |
| Legacy deletion gate | §13 | §13 | §10 | §13 | §13 | §13 | §13 | §13 |

All section references are relative to `languages/`.

## Cross-layer conclusions

- All independent plugins consume verified CoreIR and own a real dialect AST.
- Every plugin owns an exhaustive capability strategy and known-symbol/helper
  catalogue.
- Every plugin derives dependencies and file placement from typed references.
- Every independent plugin owns an opaque render-ready certificate and total
  structural renderer; JavaScript executable bytes are derived solely by the
  pinned TypeScript compiler.
- All outputs prove flat interfaces, explicit composition, first-class
  polymorphic values, determinism, and native behavior.
- Java and C++ alone may use the certified typed one-edge target adapter
  heritage form; no portable layer exposes inheritance.
- No current implementation passes this matrix. Migration state lives in
  `compliance.md`.
