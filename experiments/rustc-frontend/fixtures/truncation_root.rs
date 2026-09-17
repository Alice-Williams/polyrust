struct Sample {
    value: f64,
}
/// Trace A.
fn trace(value: f64) -> f64 {
    value
}
pub fn direct(value: f64) -> f64 {
    value.trunc()
}
pub fn associated(value: f64) -> f64 {
    f64::trunc(value)
}
pub fn local(value: f64) -> f64 {
    let saved = trace(value);
    saved.trunc()
}
pub fn imported(value: f64) -> f64 {
    truncation_middle::relay(value).trunc()
}
pub fn forwarded(value: f64) -> f64 {
    truncation_middle::integral(value)
}
pub fn shared(value: f64) -> f64 {
    let borrow = &value;
    (*borrow).trunc()
}
pub fn record(value: f64) -> f64 {
    let sample = Sample { value };
    sample.value.trunc()
}
pub fn composed(value: f64) -> f64 {
    (-truncation_middle::relay(trace(value))).trunc()
}
pub fn positive_zero() -> f64 {
    0.0_f64.trunc()
}
pub fn negative_zero() -> f64 {
    (-0.0_f64).trunc()
}
