# M35-03A-02N-01 — Independent binary64 arithmetic oracle

- Status: complete
- Parent: [binary64 arithmetic](M35-03A-02N-floating-arithmetic.md)

## Contract

Implement a test-only dependency-free integer/rational oracle for Add, Subtract,
Multiply and Divide. Decode binary64 fields exactly; round finite rational
results to nearest with ties to even. Specify signed-zero and nonfinite cases
explicitly. Never compute expected results with host floating arithmetic.
This checkpoint changes no compiler or target admission.

## Definition of done and tests

- Golden cases cover ties of both parities, exact cancellation, zero signs,
  subnormal/normal transitions, underflow, overflow, infinities and NaNs.
- Finite bit patterns round-trip through rational decoding/encoding.
- Deterministic boundary cross-products and random-bit pairs agree with actual
  pinned Rust operations for all four operators, observing NaNs by category.
- Wrong-operation, swapped-operand and zero-sign controls are non-vacuous.
- The Rust reference passes Clippy; focused Bazel and full Linux release/lint
  gates pass with no test disabled. Independent review is clean.
- Files remain focused and reusable by subsequent C/Java native proofs.

## Proof receipt

Exact reviewed implementation tree:
58e1f3c749f3bd41f92c9d4a19510ff9d907ba7e.

The Linux full Bazel release/lint gate passes all 860 test targets, invocation
1b602626-c05c-45c8-ac9e-98f20a07147a (48.359 seconds; 2 executed, 858 cached).
Rust reference Clippy and the actual pinned native comparison are included.
The initial implementation also passed the full gate before the additional
systematic rounding checks were added; no test was disabled.

Evidence: 28 golden cases, 1,068 exact finite round trips, 49,134 independently
known midpoint/quarter-neighbor checks across every normal binade and selected
subnormal boundaries, and 12,864 native Rust operator results over 3,216 operand
pairs. Wrong-operation, swapped-operand and lost-zero-sign controls all differ
from the independent expected sequence.

An independent review of the exact final tree found no core defect or required
proof gap. It checked finite encoding/decoding, carry and overflow boundaries,
nonfinite/zero tables, corpus coverage, oracle independence and Bazel wiring.
No C/Java target or compiler admission was changed. Those proofs remain in the
following checkpoints, not inferred from this test-only foundation.
