#![forbid(unsafe_code)]
fn left(value: f64) -> f64 {
    value
}
fn right(value: f64) -> f64 {
    value
}
pub fn add(a: f64, b: f64) -> f64 {
    left(a) + right(b)
}
pub fn subtract(a: f64, b: f64) -> f64 {
    left(a) - right(b)
}
pub fn multiply(a: f64, b: f64) -> f64 {
    left(a) * right(b)
}
pub fn divide(a: f64, b: f64) -> f64 {
    left(a) / right(b)
}
pub fn nested(a: f64, b: f64) -> f64 {
    (left(a) + right(b)) * right(left(a) - right(b))
}
pub fn division(a: f64, b: f64) -> f64 {
    left(a) / (right(b) / left(a))
}
pub fn borrowed(a: f64, b: f64) -> f64 {
    let left = &a;
    let right = &b;
    -*left + *right
}
pub fn literal(a: f64) -> f64 {
    left(a) * -1.0
}
pub fn negated(a: f64) -> f64 {
    -(left(a) + 1.0)
}
pub fn absolute(a: f64) -> f64 {
    (left(a) + 1.0).abs()
}
pub fn nan(a: f64) -> bool {
    (left(a) + 1.0).is_nan()
}
pub fn truncated(a: f64) -> f64 {
    (left(a) + 1.0).trunc()
}
