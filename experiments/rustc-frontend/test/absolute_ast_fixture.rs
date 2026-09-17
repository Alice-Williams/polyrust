fn input(value: f64) -> f64 {
    value
}
pub fn direct(value: f64) -> f64 {
    value.abs()
}
pub fn associated(value: f64) -> f64 {
    f64::abs(value)
}
pub fn call(value: f64) -> f64 {
    input(value).abs()
}
pub fn associated_call(value: f64) -> f64 {
    f64::abs(input(value))
}
pub fn negated(value: f64) -> f64 {
    (-input(value)).abs()
}
pub fn literal() -> f64 {
    (-0.0_f64).abs()
}
pub fn borrowed(value: f64) -> f64 {
    let borrow = &value;
    (*borrow).abs()
}
