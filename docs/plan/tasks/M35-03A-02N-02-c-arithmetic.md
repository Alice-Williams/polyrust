# M35-03A-02N-02 — C binary64 arithmetic target foundation

- Status: complete
- Parent: [binary64 arithmetic](M35-03A-02N-floating-arithmetic.md)
- Depends on: 02N-01
- Specification: [C mapping](../../specification/typed-generation/languages/c/rust-floating-arithmetic.md)

## Contract

Admit only exact F64 Add/Subtract/Multiply/Divide nodes in the existing shared C
profile. Preserve integer safety, operand authority and recursive validation.
Extend structural rendering with the four exact operator tokens; preserve fully
parenthesized expression rendering. Verify supported binary64 execution/profile
requirements before claiming native arithmetic equivalence.

## Definition of done and tests

- Actual original/importing certificates admit each operator and composition.
- Mixed types, remainder, casts, unguarded integer arithmetic, forged imports
  and malformed nodes remain rejected at the appropriate boundary.
- GCC14/Zig O0/O2 compile and execute against 02N-01 exact expectations;
  prove signed zeros, gradual underflow, overflow, nonfinite categories and
  separate-operation rounding. Wrong operator, grouping, operand order,
  dropped/duplicated calls and fused-result faults must be detected.
- Target numeric, call, stack, source-byte and dependency checks remain active.
  Pure operators introduce no math-library or runtime dependency.
- Old bundle bytes, full Linux release/lint gate and independent review pass.
  This does not admit Rust source arithmetic yet.

## Implementation and focused evidence

The checked C profile admits only exact F64 Add/Subtract/Multiply/Divide.
Structural rendering maps these four enum cases to fully parenthesized syntax;
it makes no semantic, import or evaluation-order decisions. Existing contextual
reconstruction, numeric safety, sequencing, dependency and resource walkers still
visit both operands. Ordered calls are materialized into separate full-expression
locals. Source arithmetic admission remains deferred to 02N-04.

Three target-only packages (leaf identities, arithmetic middle, forwarding root)
use original authenticated certificates. Six functions cover the four operators,
(left + right) * right and (left * right) + (-1). All owners have empty system
library closures; actual combined C/header byte lengths fit their certified
output bounds. Neither production math calls nor static runtime files are added.

- Focused tree: 37f6d7769405f8d5ef353ec55580f5d368fe9c23.
  Invocation 38665061-06c2-4132-ac3a-37f47459c916 passed all four selected tests
  in 53.838 seconds (native test 26.79 seconds).
- 3,217 input pairs produce 38,604 exact non-NaN bit/NaN-category results per
  GCC14/Zig O0/O2 run, separately compiling all three owners and the consumer.
  The shared integer/rational oracle supplies expectations independently.
- Eight compiled faults exercise wrong operator, operand reversal, zero-sign
  loss, changed grouping, contraction, dropped/duplicated calls and reordered
  full expressions. Value-preserving faults retain exact results but change
  independently expected A/B traces. Every fault is non-vacuous.
- The initial broad fma mutant exposed a Zig-linked fma result of NaN for
  minimum-subnormal * minimum-subnormal - 1. This is an unadmitted foreign
  library operation, not generated arithmetic. The contraction control now
  invokes fma only for the exact normal pair (1+2^-27, 1-2^-27); all other
  pairs retain the original expression. The rational single-round result for
  this pair differs from the baseline. This narrows only the deliberate fault,
  not any baseline arithmetic coverage.
- Mixed arithmetic, integer arithmetic, integer division by zero, float casts,
  remainder, malformed call arities and foreign-registry references reject.
- Independent review of 2d93b796305ba4ae5b219b456cabcb32fcc65c14 found no core
  correctness, security or required-proof findings. It covered recursive
  safety/resource/import checks, native mutations, byte bounds and exact-path
  test-harness policy exceptions. Per-operator sequence mutations were optional:
  all operators use the same ordered materialization path.
- Source policy classifies only the two new handwritten Python harness paths,
  with adjacent test/production path rejection controls; no broad exemptions.

The first focused run caught missing renderer cases and the first full gate
caught the new native filegroup macro's missing name argument. Both were fixed;
no tests were disabled. Final full-gate evidence is recorded at completion.

## Completion receipt

The corrected exact tree f12f04d1e62ae087068d43e5f44d823d3c48c387 passed
all 860 Linux Bazel release tests, invocation
17c71ea6-a257-4b22-b324-ccebeb1c8d9a (47.961 seconds; 3 executed,
857 cached). Rust formatting/Clippy, Bazel formatting/lint, source-policy controls,
all native arithmetic tests and separate capacity targets are included.
The preceding full run bcb0de6e-7fe6-41b6-bb05-bfb817115a7f completed in
858.421 seconds with 859 passing tests and only the now-fixed Bazel macro lint
failure. Nothing was disabled or weakened to pass.

The independent reviewer checked the macro/call-site and receipt delta on this
exact final implementation tree and found no new issue. All 138 file hashes
across 18 older compiler-generated bundles and all 19 preserved ownership-work
hashes are unchanged. Java draft work is excluded from this C checkpoint.
C target arithmetic is complete; Java target and checked Rust source admission
remain separate later checkpoints. Publishing is still blocked by the existing
push-approval decision; local passing tests do not establish remote CI status.
