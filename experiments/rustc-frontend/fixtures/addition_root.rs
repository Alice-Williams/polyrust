//! Public forwarding without copying dependency implementations.
#![forbid(unsafe_code)]
pub fn addition32(left: i32, right: i32) -> i32 {
    addition_middle::operations::addition32(left, right)
}
pub fn addition64(left: i64, right: i64) -> i64 {
    addition_middle::operations::addition64(left, right)
}
