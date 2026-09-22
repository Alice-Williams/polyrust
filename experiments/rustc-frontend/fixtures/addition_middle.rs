//! Checked wrapping addition in its original public module.
#![forbid(unsafe_code)]
/// Addition operations preserve source module identity.
pub mod operations {
    /// Exact signed 32-bit wrapping addition with ordered operands.
    pub fn addition32(left: i32, right: i32) -> i32 {
        addition_leaf::left32(left).wrapping_add(addition_leaf::right32(right))
    }
    /// Exact signed 64-bit wrapping addition with ordered operands.
    pub fn addition64(left: i64, right: i64) -> i64 {
        i64::wrapping_add(addition_leaf::left64(left), addition_leaf::right64(right))
    }
}
