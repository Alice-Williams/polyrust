fn input(value: f64) -> f64 {
    value
}
pub fn direct(value: f64) -> f64 {
    value.trunc()
}
pub fn associated(value: f64) -> f64 {
    f64::trunc(value)
}
pub fn call(value: f64) -> f64 {
    input(value).trunc()
}
pub fn associated_call(value: f64) -> f64 {
    f64::trunc(input(value))
}
pub fn negated(value: f64) -> f64 {
    (-input(value)).trunc()
}
pub fn literal() -> f64 {
    (-0.0_f64).trunc()
}
pub fn borrowed(value: f64) -> f64 {
    let borrow = &value;
    (*borrow).trunc()
}

pub fn flag(value: bool) -> bool {
    value
}
pub fn effect() {}
