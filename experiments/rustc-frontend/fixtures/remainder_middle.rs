//! Checked truncating remainder in its original public module.
#![forbid(unsafe_code)]
/// Remainder operations preserve source module identity.
pub mod operations {
    /// Truncating quotient; first operand is evaluated before the second.
    pub fn remainder(left: f64, right: f64) -> f64 {
        remainder_leaf::left(left) % remainder_leaf::right(right)
    }
}
