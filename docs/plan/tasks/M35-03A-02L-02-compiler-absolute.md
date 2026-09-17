# M35-03A-02L-02 — Checked Rust binary64 absolute-value mapping

- Status: planned
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
