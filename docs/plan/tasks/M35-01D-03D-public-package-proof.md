# M35-01D-03D — Public C package integration proof

- Status: complete
- Depends on: [M35-01D-03C](M35-01D-03C-rust-public-api-packages.md)
- Parent: [M35-01D-03](M35-01D-03-c-public-packages.md)

## Goal

Close the public-package milestone only after the complete generated artifact,
private/public boundary and historical regressions are proven together.

## Definition of done

- Generate the complete package from actual Rust through the checked compiler,
  existing C AST, shared linking/certification and structural renderer.
- Independent consumers compile separately using only public headers, with
  duplicate inclusion and varied supported include order.
- Public manifest, native symbols and documentation agree exactly with the
  compiler binding graph; private implementation artifacts remain private.
- All required child evidence is recorded; no failing test is disabled.

## Tests and proof

- Full pinned GCC/Zig O0/O2 and GCC ASan/UBSan differential matrix.
- Compiler-negative, ownership/import/manifest mutation and native resource
  gates plus Clippy, rustfmt, buildifier, docs and source-policy tests.
- Entire release/frontend/C migration gate passes in the Linux dev container.
- Fresh broad Sol Extra High review loop has no unresolved core findings.
- Local generated example byte-matches the tested Bazel artifact. Update parent
  status and continue M35-01D-04; hold pushes until C and Java migration gates pass.

## Integration test preparation

Following the clean independent 03C review, extend its existing native consumer
oracle without changing the generated implementation. Compile separate consumers
with the public header first, standard headers first, and interleaved includes;
each includes the public header twice. Run all 8,204 input vectors in each of
the eight compiler/optimization/sanitizer configurations under a 1 MiB stack.
Keep the exact production object symbol comparison and private-call rejection.
The typed-AST frame and aggregate-capacity evidence remains in 03B's full C suite.

## Evidence

- `a3f68a1f-6b51-4b51-9f05-cb5593a78996`: complete release/frontend/C,
  shared and Java gate, 345 targets and 308/308 test targets pass (178 seconds).
  Bazel reused 2,640 action-cache hits and 426 disk-cache hits; cached test results
  remained enabled. No tests were disabled.
- The native public package test runs 8,204 vectors across eight configurations
  and three include orders under a 1 MiB stack, with separate compilation and
  exact production symbol inventory. All 24 executions match native Rust.
- The visible ignored `experiments/rustc-frontend/output/public-package` example
  still byte-matches the tested declared tree artifact after the integration run.
- 03A proves linked typed file imports/guards; 03B proves exact whole-package
  reconstruction, privacy and per-file/aggregate/frame resources; 03C proves
  complete admitted compiler API mapping and publication. Their regressions are
  included here, along with the historical generated Java and real-world gates.
- Fresh broad Sol Extra High integration review found no core correctness or
  proof defects after tracing the complete 03A-D contract and native evidence.
  Documentation gate `202ef981-a8a2-49a1-8d18-6f7b53db5651` passes.
  Pushes stay held until separate-crate and Java migration gates are complete.
