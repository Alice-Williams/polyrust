//! Built-in f64 negation through values, calls, shared reads and records.
struct Sample {
    value: f64,
}
/// Trace G.
fn trace(value: f64) -> f64 {
    value
}
pub fn direct(value: f64) -> f64 {
    -value
}
pub fn nested(value: f64) -> f64 {
    -(-value)
}
pub fn local(value: f64) -> f64 {
    let saved = trace(value);
    -saved
}
pub fn imported(value: f64) -> f64 {
    -floating_middle::relay(value)
}
pub fn restored(value: f64) -> f64 {
    -floating_middle::negate(value)
}
pub fn shared(value: f64) -> f64 {
    let borrow = &value;
    -*borrow
}
pub fn record(value: f64) -> f64 {
    let sample = Sample { value };
    -sample.value
}
pub fn composed(value: f64) -> f64 {
    -floating_middle::relay(-trace(value))
}
pub fn negative_zero() -> f64 {
    -0.0
}
pub fn restored_zero() -> f64 {
    -(-0.0)
}
