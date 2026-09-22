//! Public forwarding without copying dependency implementations.
#![forbid(unsafe_code)]
pub fn multiplication32(left: i32, right: i32) -> i32 {
    multiplication_middle::operations::multiplication32(left, right)
}
pub fn multiplication64(left: i64, right: i64) -> i64 {
    multiplication_middle::operations::multiplication64(left, right)
}
