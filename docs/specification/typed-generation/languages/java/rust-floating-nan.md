# Rust f64::is_nan in Java21

- Status: implemented and verified for standard inherent f64::is_nan
- Contract: [shared](../../rust-floating-nan.md)

JavaFloatingNaN consumes private NaNInput and checks its Reader context. Lower
the original receiver once, require TypePlan::F64, and materialize one primitive
Double local. Construct JavaExprKind::Binary with NotEqual, Equality precedence,
two reads of that exact local, and primitive Boolean result. Value::new checks
the result representation. No Double wrapper or known-callable registration
is necessary, and original imported receiver calls retain their witnesses.

Java inequality against oneself is true exactly for NaN
([JLS 15.21.1](https://docs.oracle.com/javase/specs/jls/se21/html/jls-15.html#jls-15.21.1)).
Keep ordinary target certification, resource budgets and renderer unchanged.
Compile generated original-owner classes separately with Java21 -Xlint:all
-Werror. Test raw input encodings through handwritten consumers, not generated
bit conversion code; compare returned bool and receiver traces with independent
integer-mask expectations.
