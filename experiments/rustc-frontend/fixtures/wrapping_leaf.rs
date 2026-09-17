//! Ordinary public scalar library with real primitive wrapping operations.
pub fn identity32(value: i32) -> i32 {
    value
}
pub fn negate32(value: i32) -> i32 {
    value.wrapping_neg()
}
pub fn identity64(value: i64) -> i64 {
    value
}
pub fn negate64(value: i64) -> i64 {
    i64::wrapping_neg(value)
}
