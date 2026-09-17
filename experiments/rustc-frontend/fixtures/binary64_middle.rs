//! Original dependency identity survives a second generated package.
pub fn relay(value: f64) -> f64 {
    binary64_leaf::identity(value)
}
pub fn tenth() -> f64 {
    binary64_leaf::decimal_tenth()
}
