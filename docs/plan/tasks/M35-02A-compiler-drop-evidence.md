# M35-02A — Observe pinned compiler drop evidence

- Status: complete
- Parent: [M35-02](M35-02-rustc-owned-values.md)
- Depends on: M35-01E

## Contract

Build a separately cached, backend-independent probe against the existing pinned
rustc-dev toolchain. After successful full analysis, borrow the compiler-owned
drop-elaborated, pre-optimization MIR query and verify its exact phase and body
owner. Do not parse dumps, override compiler passes, call optimized MIR first,
or construct target code/ownership certificates from this observation.

Observe typed locals, moves, calls, projected drop places and control-flow/drop
edges for straight moves, conditional moves, partial moves, early returns and
shadowed names. Distinguish actual Box types by the compiler's language-item
DefId, never a spelling. Record HIR ownership and MIR source identity without
claiming variable/debug names authenticate correspondence between the two.

## Definition of done and tests

- The probe compiles under Bazel/Clippy and all sources pass Rustfmt.
- Typed assertions pin the observed MIR phase and query lifecycle, bind every
  body to its source owner, and confirm concrete allocation/drop/move shapes.
- A same-spelled user Box is not classified as the standard Box language item.
- Invalid borrow/move input fails before any success report; no renderer or
  output package is invoked. Existing C and Java adapters still reject heap input
  without creating a generated artifact.
- Every reported fixture has an explicit assertion, not a positive count over
  an otherwise unchecked aggregate. Document observations and unresolved HIR/MIR
  correspondence questions. Fresh review and complete Linux/Bazel gates pass.

## Boundary

Compiler observation is not heap translation, a new borrow checker, or proof
that generated C cleans up correctly. M35-02B/C/D own those later obligations.

## Implementation and evidence

The independent `owned_probe` executable consumes only compiler types and has
no concrete backend dependency. `owned_probe_test` additionally invokes C and
Java adapters for unsupported-heap/output-preservation controls. Typed per-body
assertions cover seven functions; valid-Rust conditional/partial/early-return
and counterfeit-name mutations must fail those assertions. The dedicated format
target includes all probe Rust modules and the input fixture.
See [exact observations](../../specification/typed-generation/languages/c/rust-drop-observations.md).

- Focused probe/format/docs/buildifier gate `4ecffc83-111a-41a9-a6f0-e2937c029b3d`:
  4/4 passed after review fixes.
- Isolated tree `4e89ecdf9f299566fa18efe23fa5e791e33cd3d3`:
  full gate `be8a6e17-8a31-405f-8535-4bb2fd9b85d7`, 375/375 tests across
  490 targets passed in 20.455 seconds. Four tests executed; other valid cached
  results were retained. Git blob bytes and executable modes were verified before
  extraction. Unfinished M34 ownership work was excluded and left unchanged.
- The gate includes release, all compiler-experiment targets, C/Java backend and
  shared codegen tests, typed compile-negative contracts, Rust lint/format,
  Bazel lint and documentation. No historical tests were disabled.

The first independent Sol Extra High review found three accepted gaps: focused
fixture-format coverage, inaccurate README test-isolation wording, and failure
to pin the counterfeit fixture's actual spelling. All were corrected; the last
also gained a rename mutation. No requested feature was substituted for a core
fix. The fresh independent Sol Extra High review found no further actionable
core findings and no optional extras warranting a hold. The final
documentation-tree gate precedes publication; its gate/tree identities are
recorded in the checkpoint commit message.
