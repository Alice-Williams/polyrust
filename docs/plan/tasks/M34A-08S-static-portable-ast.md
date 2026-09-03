# M34A-08S — Add the static valid-by-construction portable AST

- Status: in-progress
- Depends on: M34A-08R
- Blocks: M34A-10S and static migration of every remaining language

## Goal

Implement ADR-0006's typed generic authoring boundary so the Rust compiler
rejects invalid programs in the initial `StaticV1` portable profile and safe
code cannot construct or mutate a forged `StaticProgram<StaticV1>`.

## Definition of done

- `StaticProgram<F>`, `Expr<T>`, type witnesses, typed locals, functions,
  records, fields, and constructors have private representations.
- `StaticV1` exposes the exact primitive, record, function, call, field, and
  intrinsic surface specified by ADR-0006.
- Operand, result, function argument, function return, constructor argument,
  and record-field relationships are Rust generic constraints rather than
  runtime `CoreType` comparisons in the public API.
- Literal portable identifiers are checked during Rust constant evaluation;
  arbitrary unchecked strings cannot enter the static AST.
- Symbol identity is distinct from preferred spelling and final names are
  deterministic and collision-free.
- `Supports<F>` has no blanket implementation.
- A private compatibility bridge may invoke the existing checker/CoreIR path,
  but its rejection is an internal invariant defect and no unchecked value is
  exposed from `StaticProgram<F>`.
- The dynamic builder remains available under clearly distinct APIs.

## Tests

- Rust compile-pass coverage for every `StaticV1` constructor and operator.
- Rust compile-fail coverage for every mismatched relationship named above.
- Compile-fail protected-name and proof-wrapper construction/mutation cases.
- Collision tests prove repeated preferred spellings cannot produce duplicate
  portable or target declarations.
- Defensive bridge replay proves every generated static candidate passes the
  existing checker and CoreIR verifier.
- `bazel test //crates/build:all --nocache_test_results --test_output=errors`
- Rustfmt, Clippy, Buildifier, documentation, and dependency-boundary gates.

## Commit gate

Commit and push `M34A-08S: add static portable AST` only after all focused
proof passes in the Linux development container. Record the exact local and
remote SHA and require hosted CI before claiming the shared static layer
complete.

## Exit evidence

Pending implementation.

