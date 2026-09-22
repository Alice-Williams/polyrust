//! Checked signed widening in its original public module.
#![forbid(unsafe_code)]
/// Widening preserves source module identity.
pub mod operations {
    /// Preserve the exact signed value across widths.
    pub fn widen(value: i32) -> i64 {
        widening_leaf::input(value) as i64
    }
}
