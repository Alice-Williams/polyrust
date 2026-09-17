//! Ordinary Rust operators, checked before runtime-free target lowering.
#![forbid(unsafe_code)]

/// Arithmetic remains in its original public Rust module.
pub mod operations {
    /// Preserve first-then-second source evaluation.
    pub fn add(left: f64, right: f64) -> f64 {
        arithmetic_leaf::left(left) + arithmetic_leaf::right(right)
    }
    pub fn subtract(left: f64, right: f64) -> f64 {
        arithmetic_leaf::left(left) - arithmetic_leaf::right(right)
    }
    pub fn multiply(left: f64, right: f64) -> f64 {
        arithmetic_leaf::left(left) * arithmetic_leaf::right(right)
    }
    pub fn divide(left: f64, right: f64) -> f64 {
        arithmetic_leaf::left(left) / arithmetic_leaf::right(right)
    }
    /// The addition rounds before multiplication.
    pub fn grouped(left: f64, right: f64) -> f64 {
        let left = arithmetic_leaf::left(left);
        let right = arithmetic_leaf::right(right);
        (left + right) * right
    }
    /// This is deliberately not fused multiply-add.
    pub fn separate(left: f64, right: f64) -> f64 {
        let left = arithmetic_leaf::left(left);
        let right = arithmetic_leaf::right(right);
        (left * right) + (-1.0)
    }
    /// Right-hand nesting cannot be reassociated into left-hand division.
    pub fn nested_division(left: f64, right: f64) -> f64 {
        let left = arithmetic_leaf::left(left);
        let right = arithmetic_leaf::right(right);
        left / (right / left)
    }
}
