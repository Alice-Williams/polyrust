//! Checked wrapping subtraction in its original public module.
#![forbid(unsafe_code)]
/// Subtraction operations preserve source module identity.
pub mod operations {
    /// Exact signed 32-bit wrapping subtraction with ordered operands.
    pub fn subtraction32(left: i32, right: i32) -> i32 {
        subtraction_leaf::left32(left).wrapping_sub(subtraction_leaf::right32(right))
    }
    /// Exact signed 64-bit wrapping subtraction with ordered operands.
    pub fn subtraction64(left: i64, right: i64) -> i64 {
        i64::wrapping_sub(
            subtraction_leaf::left64(left),
            subtraction_leaf::right64(right),
        )
    }
}
