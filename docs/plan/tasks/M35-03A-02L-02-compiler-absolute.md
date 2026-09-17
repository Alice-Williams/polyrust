# M35-03A-02L-02 — Checked Rust binary64 absolute-value mapping

- Status: complete
- Parent: [absolute value](M35-03A-02L-floating-absolute.md)
- Depends on: [target conditional foundation](M35-03A-02L-01-floating-conditional.md)

## Contract

Authenticate canonical original checked f64::abs method/associated calls using
the standard core inherent method identity, nongeneric safe Rust (f64)->f64
signature and unadjusted receiver/result. Register an executable
FloatingAbsolute capability with private compiler input. Ordinary free
functions named abs remain ordinary calls. Both call inventories discover the
primitive but continue walking its original receiver for imported call authority.

Materialize the receiver once. Form the typed expression
receiver == +0.0 ? +0.0 : (receiver < +0.0 ? -receiver : receiver).
Every comparison reads the same exact F64/Double local. C comparisons normalize
Int to Bool, conditional branch/result types remain exact F64. Java uses
primitive Double and correct Equality/Relational/Unary/Conditional precedence.
The renderer performs only structural emission.

## Definition of done and tests

- Exact non-NaN sign-clear integer oracle, including both zeros/subnormals/
  infinities; NaN category-only contract unchanged.
- Original Rust and three-crate generated C/Java agree; method/associated,
  ordinary same-name free calls, imported receivers, local/field/shared reads
  and compositions are covered. Export actual examples outside Docker.
- Read-only AST and canonical/context probes establish the complete conditional
  shape, original receiver dataflow, positive-zero literal and exact types.
  Retained-call disconnected-result, missing zero case, wrong sign/condition
  and dropped/duplicated receiver faults must fail their appropriate oracles.
- Independent missing/duplicate/wrong capability/context/input/output/private
  compiler contracts and atomic unsupported-neighbor rejection remain enforced.
- Prior bundles unchanged, full Linux release/lint tests and fresh independent
  review pass before a scoped commit/push. Only partial parity is recorded.

## Implementation and evidence

- Exact implementation tree 286e29bd096624b75d064161981f5d02c17bbc6b passed
  Linux Bazel test //... //:release_gate: 840/840 tests across 1,280 targets,
  3 executed and 837 cached, 74.100 seconds.
  Invocation: 820aa51f-0cb8-4758-a271-4977d71304ad.
- Both builders have a required executable FloatingAbsolute slot with exact
  Reader/input/output bounds. Existing missing-capability controls explicitly
  install it so they retain their original independent failure cause.
- Native proof compares 578 results across 72 raw binary64 inputs and two
  literal results, three original Rust crates, GCC14/Zig C17 O0/O2 and Java21
  strict lint. Seven value/trace faults are detected. Standard abs and a
  same-named ordinary free function remain distinct: exact middle import IDs,
  leaf B/root A traces and value-preserving ordinary-call replacement/duplicate
  controls close the review-discovered non-vacuity gap.
- Each target passes seven read-only AST observations, canonical/context/
  copied-HIR controls and two retained-call disconnected-result controls.
  Observed and production output bytes are identical. Fourteen independent
  compile-negative contracts and 48 atomic unsupported-source cases pass.
- The first full gate caught an older i32::abs rejection expecting the generic
  unsupported-expression diagnostic. Its unchanged source now correctly
  expects the new exact f64-only primitive diagnostic; no tests were disabled.
- All 138 files in the earlier 18 bundle baseline and both preceding
  floating-negation and NaN-classification pairs are byte-identical.
  Twenty-four actual example files are exported at
  generated/examples/floating-absolute-286e29bd; generated C/Java directories
  are byte-identical to the tested unmodified bundles.
- Sol Extra High library_import_review accepted the ordinary-call proof fix
  and found no other core or required-proof findings. Fresh independent
  binary64_targets_review also found no further core/proof issue. Its inventory
  classification observation is corrected: legacy FloatAbs belongs to
  JavaFloatingPointInspection, not JavaFloatingPointArithmetic.
- Optional extra source spellings/nested abs cases and more stage-specific
  diagnostics are deferred: canonical method/associated identity, composition,
  exact AST/dataflow and independent failure contracts already cover this
  bounded operation. No NaN sign/payload, general arithmetic, full floating
  inspection or runtime-retirement claim is made.
