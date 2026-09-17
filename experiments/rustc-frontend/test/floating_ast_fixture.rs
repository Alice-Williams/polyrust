//! Small closed source inventory for non-vacuous read-only AST probes.
pub fn input(value: f64) -> f64 {
    value
}
pub fn direct(value: f64) -> f64 {
    -value
}
pub fn nested(value: f64) -> f64 {
    -(-value)
}
pub fn call(value: f64) -> f64 {
    -input(value)
}
pub fn nested_call(value: f64) -> f64 {
    -input(-input(value))
}
pub fn literal() -> f64 {
    -(-0.0)
}
pub fn borrowed(value: f64) -> f64 {
    let borrow = &value;
    -*borrow
}
