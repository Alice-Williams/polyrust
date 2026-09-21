//! Numeric behavior of Apache-2.0 stdlib is-negative-zero 0.2.3.
#![forbid(unsafe_code)]
/// Return true only for IEEE-754 binary64 negative zero.
pub fn is_negative_zero(value: f64) -> bool {
    value == 0.0 && reciprocal(value) < 0.0
}
fn reciprocal(value: f64) -> f64 {
    1.0 / value
}
