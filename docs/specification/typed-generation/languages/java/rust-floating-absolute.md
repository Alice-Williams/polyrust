# Rust f64::abs in Java21

- Status: bounded implementation verified; see M35-03A-02L-02 evidence
- Contract: [shared](../../rust-floating-absolute.md)

## Certified target foundation

Use the existing JavaExprKind::Conditional and normal verifier/linker/
certificate path. Boolean condition, exact primitive Double branches/result
and Conditional precedence must be checked; malformed types/precedence and
counterfeit imported authority reject. The closed dependency body profile explicitly
requires Conditional precedence; the general expression verifier already checks
condition, branch and result types.

## Checked source mapping

JavaFloatingAbsolute validates the private compiler witness, lowers and
materializes its original receiver once, requires TypePlan::F64, and constructs
the shared zero/negative selection shape. Use checked primitive Double positive
zero; Equality and Relational comparison precedence with Boolean results;
Unary negation precedence; nested Conditional nodes with exact Double type.
Both comparisons and result branches reuse the same original local. Value::new
checks the final representation. No Math/Double import, wrapper or Runtime
class is emitted.

## Proof

Separately compile original-owner generated classes with Java21 -Xlint:all
-Werror. Handwritten raw-bit consumers compare exact non-NaN sign-clear
expectations and NaN categories, including signed zeros and subnormals. Native
call traces and exact AST reconstruction/detachment controls prove the receiver
is neither skipped, duplicated nor disconnected. Existing classfile/source
budgets and full release/lint gates remain enabled.
