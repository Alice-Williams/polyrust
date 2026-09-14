# M35 — Rust compiler frontend proof

- Status: in-progress
- Depends on: M00 and the existing typed-generation baseline
- Takes priority over unfinished M34A-11 ownership analysis

## Outcome

Compile ordinary Rust through rustc, reuse its parser, type checker and borrow
checker, and translate a deliberately small admitted HIR subset to C17.
Compare native Rust and generated C on identical input vectors.

## Ordered tasks

1. [M35-01 — Compiler adapter and native proof](../tasks/M35-01-rustc-adapter.md).
   - [M35-01A — Compiler boundary review repairs](../tasks/M35-01A-compiler-boundary-repairs.md) blocks its completion.
2. [M35-01B — Existing-type C HIR bridge](../tasks/M35-01B-c-hir-typed-bridge.md).
3. [M35-01C — Documentation attributes](../tasks/M35-01C-c-hir-documentation.md).
4. [M35-01D — Crate and API boundaries](../tasks/M35-01D-c-files-and-visibility.md).
5. [M35-01E — Java compiler-frontend retrofit](../tasks/M35-01E-java-rustc-retrofit.md).
6. [M35-02 — Owned allocation and cleanup proof](../tasks/M35-02-rustc-owned-values.md).
7. [M35-03 — Cross-language production integration decision](../tasks/M35-03-rustc-integration.md).

Implement the C path first, then retrofit Java using its existing typed AST.
Keep tests enabled and hold pushes until the full migration gate is green.

## Definition of done

The adapter consumes compiler data structures after successful analysis, never
our own parsed source or pretty-printed compiler dumps. Invalid Rust and unsupported Rust have
separate diagnostics and produce no output. Each task records exact
Linux/Bazel gates, limitations and independent review.

The first proof covers a non-Copy struct, move, shared borrow and branch.
It makes no heap-cleanup claim. Owned allocation and cleanup require M35-02
before replacing production ownership checks.

## Existing work

Committed C analysis remains historical evidence. Uncommitted child-graph work
is preserved and excluded from this experiment's commits. M34A-11's incomplete
analysis sequence is suspended pending M35-03. Planning does not mark any
unfinished task complete or remove its correctness obligations.
