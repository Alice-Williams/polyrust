# Checked Rust binary64 absolute value

- Status: bounded implementation verified; see M35-03A-02L-02 evidence
- Plan: [M35-03A-02L](../../plan/tasks/M35-03A-02L-floating-absolute.md)
- Targets: [C17](languages/c/rust-floating-absolute.md), [Java21](languages/java/rust-floating-absolute.md)

Authenticate the actual standard inherent f64::abs method or associated call,
never a string-spelled lookalike. Require canonical HIR, original TypeckResults,
nongeneric safe Rust (f64)->f64 signature and no adjustments. FloatingAbsolute
has a private compiler input and an executable mapping in each consuming builder.
Call discovery retains nested original-owner receiver calls.

For every non-NaN value, the result is the exact magnitude: clear the sign bit,
including negative zero, negative subnormals and negative infinity. NaN input
produces NaN. This follows the
[Rust API](https://doc.rust-lang.org/std/primitive.f64.html#method.abs) within the
existing [category-only NaN observation boundary](rust-binary64-values.md).
No claim of NaN payload/sign equivalence, floating-status-flag equivalence or
arbitrary target environment support is made. Bit/sign observations remain
unsupported and require strengthening this contract before admission.

The mappings materialize the receiver once, then build:
receiver == +0.0 ? +0.0 : (receiver < +0.0 ? -receiver : receiver).
The explicit equality branch matters: merely returning negative ? -value :
value preserves -0.0 incorrectly. Both zeros must produce +0.0. All operands
are typed local reads or checked literal/unary/conditional nodes, not source
strings. No copied/custom runtime or general floating arithmetic is needed.

The target foundation proves exact floating conditionals before the compiler
adapter uses them. Integer-bit magnitude oracles, native signed-zero/NaN
boundaries, original receiver AST dataflow, once-only call traces and deliberate
missing-zero/wrong-condition/dropped/duplicated faults are required. Unsupported
neighboring source forms reject atomically; examples must be actual generated
files. Each implementation checkpoint needs full Linux release/lint gates and
fresh independent review.
