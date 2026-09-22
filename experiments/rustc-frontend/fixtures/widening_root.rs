//! Public forwarding without copying dependency implementations.
#![forbid(unsafe_code)]
pub fn widen(value: i32) -> i64 {
    widening_middle::operations::widen(value)
}
