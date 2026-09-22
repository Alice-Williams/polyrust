//! Checked wrapping multiplication in its original public module.
#![forbid(unsafe_code)]
/// Multiplication operations preserve source module identity.
pub mod operations {
    /// Exact signed 32-bit wrapping multiplication with ordered operands.
    pub fn multiplication32(left: i32, right: i32) -> i32 {
        multiplication_leaf::left32(left).wrapping_mul(multiplication_leaf::right32(right))
    }
    /// Exact signed 64-bit wrapping multiplication with ordered operands.
    pub fn multiplication64(left: i64, right: i64) -> i64 {
        i64::wrapping_mul(
            multiplication_leaf::left64(left),
            multiplication_leaf::right64(right),
        )
    }
}
