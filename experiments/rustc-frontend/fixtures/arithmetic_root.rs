//! Public wrappers consume the original arithmetic owner, not copied helpers.
#![forbid(unsafe_code)]

pub fn add(left: f64, right: f64) -> f64 {
    arithmetic_middle::operations::add(left, right)
}
pub fn subtract(left: f64, right: f64) -> f64 {
    arithmetic_middle::operations::subtract(left, right)
}
pub fn multiply(left: f64, right: f64) -> f64 {
    arithmetic_middle::operations::multiply(left, right)
}
pub fn divide(left: f64, right: f64) -> f64 {
    arithmetic_middle::operations::divide(left, right)
}
pub fn grouped(left: f64, right: f64) -> f64 {
    arithmetic_middle::operations::grouped(left, right)
}
pub fn separate(left: f64, right: f64) -> f64 {
    arithmetic_middle::operations::separate(left, right)
}
pub fn nested_division(left: f64, right: f64) -> f64 {
    arithmetic_middle::operations::nested_division(left, right)
}
