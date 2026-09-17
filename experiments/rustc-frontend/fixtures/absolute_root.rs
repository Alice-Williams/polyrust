struct Sample {
    value: f64,
}
/// Trace A.
fn trace(value: f64) -> f64 {
    value
}
pub fn direct(value: f64) -> f64 {
    value.abs()
}
pub fn associated(value: f64) -> f64 {
    f64::abs(value)
}
pub fn local(value: f64) -> f64 {
    let saved = trace(value);
    saved.abs()
}
pub fn imported(value: f64) -> f64 {
    absolute_middle::relay(value).abs()
}
pub fn forwarded(value: f64) -> f64 {
    absolute_middle::magnitude(value)
}
pub fn shared(value: f64) -> f64 {
    let borrow = &value;
    (*borrow).abs()
}
pub fn record(value: f64) -> f64 {
    let sample = Sample { value };
    sample.value.abs()
}
pub fn composed(value: f64) -> f64 {
    (-absolute_middle::relay(trace(value))).abs()
}
pub fn positive_zero() -> f64 {
    0.0_f64.abs()
}
pub fn negative_zero() -> f64 {
    (-0.0_f64).abs()
}
