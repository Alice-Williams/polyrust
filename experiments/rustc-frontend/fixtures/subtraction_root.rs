//! Public forwarding without copying dependency implementations.
#![forbid(unsafe_code)]
pub fn subtraction32(left: i32, right: i32) -> i32 {
    subtraction_middle::operations::subtraction32(left, right)
}
pub fn subtraction64(left: i64, right: i64) -> i64 {
    subtraction_middle::operations::subtraction64(left, right)
}
