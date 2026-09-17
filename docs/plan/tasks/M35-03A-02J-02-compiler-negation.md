# M35-03A-02J-02 — Checked Rust floating-negation capability

- Status: planned
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
