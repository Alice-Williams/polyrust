//! Exact primitive transport, with no generated runtime.
pub fn identity(value: f64) -> f64 {
    value
}
pub fn positive_zero() -> f64 {
    0.0
}
pub fn negative_zero() -> f64 {
    -0.0
}
pub fn minimum_subnormal() -> f64 {
    5e-324
}
pub fn maximum_subnormal() -> f64 {
    2.225073858507201e-308
}
pub fn minimum_normal() -> f64 {
    2.2250738585072014e-308
}
pub fn maximum_finite() -> f64 {
    1.7976931348623157e308
}
pub fn negative_maximum() -> f64 {
    -1.7976931348623157e308
}
pub fn next_above_one() -> f64 {
    1.0000000000000002
}
pub fn rounded_integer() -> f64 {
    9_007_199_254_740_993.0
}
pub fn decimal_tenth() -> f64 {
    0.1
}
pub fn negative_underflow() -> f64 {
    -1e-400
}
