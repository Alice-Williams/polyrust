fn input(value: f64) -> f64 {
    value
}
pub fn direct(value: f64) -> bool {
    value.is_nan()
}
pub fn associated(value: f64) -> bool {
    f64::is_nan(value)
}
pub fn call(value: f64) -> bool {
    input(value).is_nan()
}
pub fn associated_call(value: f64) -> bool {
    f64::is_nan(input(value))
}
pub fn negated(value: f64) -> bool {
    (-input(value)).is_nan()
}
pub fn literal() -> bool {
    (-0.0_f64).is_nan()
}
pub fn borrowed(value: f64) -> bool {
    let borrow = &value;
    (*borrow).is_nan()
}
