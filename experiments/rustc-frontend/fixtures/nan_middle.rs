pub fn relay(value: f64) -> f64 {
    nan_leaf::is_nan(value)
}
pub fn classify(value: f64) -> bool {
    nan_leaf::classify(value)
}
