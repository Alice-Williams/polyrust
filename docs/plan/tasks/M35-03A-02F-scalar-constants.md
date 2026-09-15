# M35-03A-02F — Scalar constant parity

- Status: in-progress
- Parent: [M35-03A-02](M35-03A-02-scalar-parity.md)
- Depends on: M35-03A-02E

## Contract

Use rustc to evaluate resolved scalar constants; never infer a constant by its
spelling or add a target-side interpreter. Preserve declaration provenance and
public export behavior. Constant evaluation is not permission to enable
unsupported runtime arithmetic, arbitrary casts, storage or generic source.

## Ordered work and definition of done

1. [02F-01 — Checked scalar constant reads](M35-03A-02F-01-constant-reads.md) — complete.
2. [02F-02 — Public and local constant declarations](M35-03A-02F-02-constant-declarations.md) — planned.

Both steps need native Rust/C/Java equality, exact typed AST evidence, atomic
rejection, resource/identity checks, independent review and isolated green
Bazel/lint gates. Do not claim complete JavaConstants catalogue parity from
private constant folding alone. Wider constant types follow their owning type
capabilities; this family initially concerns bool/i32/i64.
