# M35-03A-02N-04 — Checked Rust binary64 arithmetic

- Status: complete
- Parent: [binary64 arithmetic](M35-03A-02N-floating-arithmetic.md)
- Depends on: 02N-02 and 02N-03

## Contract

Add a private canonical-HIR ArithmeticInput and closed FloatingArithmeticOperator
enum with Add, Subtract, Multiply and Divide. Prove original typeck, built-in
operator identity, exact f64 operands/result and absent adjustments.
FloatingArithmetic must be a real executable builder slot for both targets.
Lower/materialize left then right exactly once; preserve AST grouping.

## Definition of done and tests

Three original Rust crates generate ordinary C/Java packages with correct
visibility, original imports and deterministic metadata. Compare native outputs
to Rust and 02N-01; verify traces independently with compiled fault controls.
Exact typed AST/dataflow and canonical-witness probes pass, including copied
HIR/wrong context negatives. Missing/duplicate/wrong capability contracts fail
with exactly their intended compile diagnostic; backend-free positive probes
remain complete. Unsupported widths, overloads, assignments, casts, remainder
and fused methods reject atomically. Export real generated examples and retain
old bundle hashes. Full release/lint gates and fresh independent review pass.

## Implemented evidence

The compiler-owned ArithmeticInput requires the exact canonical HIR expression,
original owning TypeckResults, built-in operator identity, exact f64
operands/result and no adjustments. Both 26-slot typed builders require its
executable mapping. Each target materializes the complete left operand before
starting the right, then constructs ordinary structural binary operators.

- Three original source crates, 17 function identities, a public nested module
  and one private helper generate ordinary C/Java packages with authenticated
  original imports and exact visibility/signatures. Pure arithmetic introduces
  no runtime, custom helper, math header or system-library dependency.
- 3,217 input pairs yield 45,038 observations per target execution. GCC14/Zig
  O0/O2 and Java21 strict lint use separate compilation for each owner/client.
  Actual Rust source results agree with the independent integer/rational oracle.
- Nine compiled test-copy faults detect wrong operators, operand reversal,
  zero-sign loss, two grouping errors, FMA contraction and dropped, duplicated
  or reordered calls. The last three preserve every value but change traces.
- Fifteen AST observations per target check canonical source/context, copied-node
  and wrong-context rejection, exact operator/type/precedence, expanded dataflow
  and original per-operand call order. Every observation detects a changed
  actual operator and a disconnected actual final-right temporary while all
  original calls remain. Probe and production bytes are identical.
- Strict native composition clients cover arithmetic inside negation, abs,
  is_nan and trunc. Only trunc needs the previously certified C math library.
- Fourteen compile-negative controls each require one intended error.
  The backend-free positive source contract remains complete. Forty valid
  unsupported-source cases reject atomically, including existing sentinels.
- All twenty historical C/Java omission chains now omit exactly one intended
  original slot. A dedicated audit follows the actual slot inventory and
  detects four deliberate omission/duplicate drift faults.

## Release evidence and repairs

First full tree d49054f20fea1e28196855a13a6dd270b60c88d8 passed 877 tests;
one old Java missing-slot fixture failed to build (unimported new mapping).
Inspection also found that fixture omitted older floating slots, weakening
its original omission proof. It now registers every non-target slot exactly
once; no expected error code/count was relaxed. Native composition wiring
was corrected to use real metadata bundles rather than expecting manifests
from the direct Java single-file adapter.

Self-review and independent review identified the initial AST mutation control
as too weak because it changed reconstructed expectations. The replacement
mutates actual generated nodes/temporary dataflow and requires all fifteen
detachment observations per language; this is proof work, not broader syntax.

Repaired exact tree d50dd04ac13798290a12c29f7fef8ae933e75850 passed all 879
Linux Bazel release/lint test targets, invocation
f56ddcd3-dda5-4618-bc81-d1d5e137a7a2 (83.413 seconds; 7 executed,
872 cached). Rust formatting/Clippy, Bazel formatting/lint, native compilation,
source policy and all historical tests remain enabled. All 138 file hashes
across 18 older bundles and all 19 preserved ownership-work hashes match.

Actual unmodified examples, baseline clients and original Rust inputs are
exported by the native target. The inspected local copy is
generated/examples/floating-arithmetic-d50dd04a/README.md (ignored, not committed).
Two independent Sol Extra High reviewers found no remaining core defect or
required-proof gap in the repaired exact tree. Both explicitly confirmed the
missing-slot and actual-dataflow repairs. Documentation-only closure is gated
again before committing this checkpoint.
Publishing remains blocked by the prior push-approval decision; these are local
test receipts, not a claim that unpublished changes passed remote GitHub CI.
