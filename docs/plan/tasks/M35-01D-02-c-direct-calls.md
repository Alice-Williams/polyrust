# M35-01D-02 — Direct calls and private same-crate helpers

- Status: complete
- Depends on: [M35-01D-01](M35-01D-01-rust-api-inventory.md)
- Parent: [M35-01D](M35-01D-c-files-and-visibility.md)
- Contract: [direct calls](../../specification/typed-generation/languages/c/rust-hir-direct-calls.md)

## Goal

Translate an admitted same-crate function graph using resolved callable
identities and the existing C function/call/statement types.

## Definition of done

- Register admitted signatures before lowering bodies. Resolve source calls by
  compiler declaration identity, never function-name matching.
- Add an executable direct-call capability binding with source-only input,
  typed C context/output and compile-negative registration contracts.
- Preserve Rust argument evaluation order with checked temporaries as needed;
  retain scope/initialization/storage checks, not unchecked C argument strings.
- Derive internal linkage for private/restricted helpers and external linkage
  only for the admitted API. Prototypes and definitions must agree.
- Keep the scope scalar/no-heap. Reject indirect calls, unimplemented signatures
  and cycles until their explicit resource/semantic contracts exist.
- Extend resource admission for the actual acyclic call graph. The old no-call
  frame estimate must not certify recursive or unbounded call paths.

## Tests and proof

- Multi-file and inline-module fixtures with identical helper names, nested
  calls, source-ordered arguments and restricted visibility execute identically
  in native Rust and strict C at O0/O2 and the sanitizer/compiler matrix.
- Typed mutations reject signature/owner/linkage mismatches, unavailable calls,
  missing prototypes, unresolved graph edges and over-budget call paths.
- Private functions have local native symbol-table entries; public inventory
  excludes test wrappers. Keep public-header proof pending M35-01D-03.
- All registration compile-fail, style, independent-review and release gates pass.

## Ordered implementation checkpoints

1. Derive closed scalar-call evidence from actual C definitions. Integrate it
   into storage checking with direct/indirect, missing body, recursive and
   effectful-body negative controls. Keep rendering admission closed while the
   resource rules are incomplete.
2. Extend C graph admission, linkage projection and structural spelling. Derive
   worst-path frame accounting from actual direct edges; prove native bounds
   before the renderer accepts the new profile.
3. Register the HIR direct-call mapping and helper signatures before bodies.
   Add source-ordered temporaries and deterministic resolved helper identities.
4. Run compiler-negative contracts, exact target mutations, native differential
   and symbol-table proofs, then fresh independent review and the full gate.

Each checkpoint keeps files focused. Intermediate diagnostic support is not
reported as end-to-end call support before all four checkpoints pass.

## Local checkpoint evidence

- Checkpoint 1: private definition-derived scalar-call evidence and its
  integration into storage checking passed seven focused Bazel targets in
  `704ee8a2-3091-460d-a77f-a897567a56a4`, including ordinary C tests,
  compile-negative contracts, source policies and mandatory linters. Independent
  Sol Extra High review found no remaining core errors in this scoped change.
- Checkpoint 2: actual direct edges, internal/external linkage projection,
  structural call rendering and per-function/worst-path admission are integrated.
  Six focused targets passed in `4999b40a-702f-4beb-b3eb-8d954fa7c3d8`:
  643 ordinary C unit tests, the exact partition contract, native call proofs,
  Rust Clippy, rustfmt and buildifier. The native proof covered 13 cases at
  GCC/Zig O0/O2 and GCC ASan/UBSan O0/O2, including all three scalar types and
  zero/three/127-argument helpers. The measured chain boundary was 54 functions
  at 1,035,712 estimated bytes; 55 rejected at 1,061,952 bytes. These counts
  describe this exact fixture, not universal limits on function count.
- Checkpoint 2 review found two native-proof gaps: optimizer-created frame rows
  were ignored, and failure to set the process stack limit could be masked.
  Both are fixed, with narrow derivative accounting and a shared fail-closed
  stack launcher plus failure injections. Fresh Sol Extra High review found no
  remaining core issues. Final launcher/native regression evidence passed in
  the full gate below, including the old no-call launcher regression.
- Checkpoint 3: helper signature and direct-call bindings,
  complete signature preregistration, private linkage and source-ordered typed
  temporaries are implemented. `bf31bfb1-1836-490e-98fc-c8fc9f16041b` passed
  17 targets, including the 8,204-input multi-module Rust/C native matrix,
  ten-slot compiler contracts, corrected 14-case C native call matrix and
  mandatory linters. Structural call-order, rejection and symbol proofs then
  passed with every compiler-frontend target in
  `a5857689-e64b-4fe0-bf61-a2de22ba5702` (47 tests and 33 build targets).
- Checkpoint 4: fresh Sol Extra High HIR review found no core correctness or
  required-proof gap. Full Linux/Bazel release/frontend/C migration gate
  `64d4910a-b7bd-41d2-ba8b-14c1637be785` passed all 302 test targets and
  33 build targets; 47 tests executed and the remainder used valid cached
  results. No tests were disabled. All five C capacity partitions were included.

## Completed proof and handoff

The source fixture includes same-spelled module helpers, a resolved import
alias, zero/one/three-parameter and mixed i32/bool signatures, shared record
storage, reversed source field order, nested calls and branch-local calls.
The compiler AST probe independently verifies call order and temporary
dominance, exact origins, private linkage, out-of-line file provenance and
three-render determinism. Native Rust/C parity covers 8,204 inputs on pinned
GCC/Zig and GCC ASan/UBSan, each at O0/O2. Separate symbol tests prove only the
Rust-public entry is exported and all five helpers have local O0 definitions.
Unsupported valid-Rust call shapes fail after analysis without creating or
overwriting output; illegal Rust remains the compiler's responsibility.

The C native resource oracle covers 14 fixtures across eight toolchain/mode
combinations, including the actual last-admitted chain, high-arity calls,
composed locals and repeated calls. Compiler derivatives are counted and the
effective controlled stack limit is checked before execution.

The tested generated example is available locally at
`experiments/rustc-frontend/output/direct_calls.c`, byte-compared with the Bazel
artifact and intentionally ignored by Git. This is same-crate scalar call
support, not public-header or separate-crate completion. Continue with
[M35-01D-03](M35-01D-03-c-public-packages.md). Java retrofit and the combined
migration push gate remain pending; no push was made for this local completion.
