struct Sample {
    value: f64,
}
/// Trace N.
fn trace(value: f64) -> f64 {
    value
}
pub fn direct(value: f64) -> bool {
    value.is_nan()
}
pub fn associated(value: f64) -> bool {
    f64::is_nan(value)
}
pub fn local(value: f64) -> bool {
    let saved = trace(value);
    saved.is_nan()
}
pub fn imported(value: f64) -> bool {
    nan_middle::relay(value).is_nan()
}
pub fn forwarded(value: f64) -> bool {
    nan_middle::classify(value)
}
pub fn shared(value: f64) -> bool {
    let borrow = &value;
    (*borrow).is_nan()
}
pub fn record(value: f64) -> bool {
    let sample = Sample { value };
    sample.value.is_nan()
}
pub fn composed(value: f64) -> bool {
    (-nan_middle::relay(trace(value))).is_nan()
}
pub fn positive_zero() -> bool {
    0.0_f64.is_nan()
}
pub fn negative_zero() -> bool {
    (-0.0_f64).is_nan()
}
