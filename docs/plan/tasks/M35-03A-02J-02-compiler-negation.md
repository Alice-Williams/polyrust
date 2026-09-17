# M35-03A-02J-02 — Checked Rust floating-negation capability

- Status: complete
- Parent: [floating negation](M35-03A-02J-floating-negation.md)
- Depends on: [target proof](M35-03A-02J-01-target-negation.md)

## Contract

Register an executable FloatingNegation capability with a private compiler
witness retaining the original HIR node, f64 operand and original type-check
context. Admit built-in unary minus with exact f64 result/operand, no overload
and no adjustments. C and Java mapping methods consume that witness, lower the
operand once and construct their existing primitive unary nodes. Preserve the
finite-negative-literal path and original dependency certificates.

## Definition of done and tests

- Builder inference requires the real mapping; missing/duplicate/wrong
  capability/context/input/output mappings and forged witnesses fail exact
  compile contracts alongside positive controls.
- Rust/three-crate generated C/Java values agree with an independent sign-bit
  oracle on signed zeros, normals, subnormals, infinities and NaN categories.
  Original operand calls occur once; dropped/duplicated calls fail trace checks.
- Read-only probes establish canonical HIR, exact target unary type/precedence,
  operand identity/placement and identical probe/production output.
- Unsupported f32, overloaded/reference negation, casts, arithmetic, floating
  constants and nonfinite literal construction reject atomically where valid
  Rust; existing signed integer negation remains separately guarded.
- Real examples are exported outside Docker. Full Linux release/lint gates and
  a fresh independent review pass, documentation records evidence, then
  commit/push. No general floating arithmetic or legacy retirement is claimed.

## Review decisions and proof evidence

- The first full gate found three backend-free probe build failures caused by
  the probe not referencing FloatingNegation/FloatingInput. Added the new
  capability and witness-method signatures to the existing backend-free probe;
  retained strict Clippy and exact compile-negative error checks.
- Accepted the independent review's call-result dataflow finding. Call presence
  and count alone do not prove that a call's result feeds negation when fixtures
  use identity functions. New separate C/Java dataflow modules reconstruct the
  checked source operand with original callee/argument/unary identities and
  compare it to the target operand after bounded, unique temporary substitution.
  Two deliberate disconnected-result controls per target leave every actual
  call in place and change only the last copy; the dataflow oracle rejects both.
- Updated the normative C capability table for FloatingNegation and the existing
  finite-f64 LiteralValues support. No production relaxation was needed.
- Exact corrected implementation tree: 416f307a23d43bba675a754b9898389180a7b2fb.
  Linux Bazel test //... //:release_gate passed 804/804 test targets across
  1,222 targets (5 executed, 799 cached), 38.925 seconds;
  invocation 223aa9e2-187d-4946-ac6d-f767fda69ead.
- Native proof: 578 results from 72 binary64 inputs, eight unary/call/read
  compositions and two signed-zero literal results, across three source crates.
  Compare original Rust, separately compiled GCC14/Zig O0/O2 and strict Java21
  with independent integer sign/category expectations. No-negation, zero-minus,
  dropped-call and duplicate-call test copies fail their exact value/trace oracles.
- Read-only probes observe eight canonical inputs/typed ASTs plus two detachment
  controls per target; output bytes equal uninstrumented production. Fourteen
  mapping/input compile contracts and 36 atomic source rejections pass.
- All 138 generated files across 18 prior bundles remain byte-identical.
  Twenty-four actual generated/source/client files are exported to the ignored
  host folder generated/examples/floating-negation-12eab814, including README.md.
- Sol Extra High library_import_review found no remaining core/required-proof
  findings after the correction. Fresh Sol Extra High binary64_targets_review
  independently reviewed the full corrected tree and found no core or required-proof
  findings. Optional imported-call detachment and record/local AST fixture expansion
  are deferred: original-owner inventory/native checks already cover foreign-call
  authority, and the native matrix covers record/local semantics. Neither expands
  the admitted contract or closes a missing required proof.
