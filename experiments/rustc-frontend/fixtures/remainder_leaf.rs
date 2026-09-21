//! Original operand producers, with a private implementation detail.
#![forbid(unsafe_code)]
/// First operand retains its original producer.
pub fn left(value: f64) -> f64 {
    identity(value)
}
/// Second operand retains its original producer.
pub fn right(value: f64) -> f64 {
    identity(value)
}
fn identity(value: f64) -> f64 {
    value
}
