//! Original signed operand producers and private implementation details.
#![forbid(unsafe_code)]
fn hidden32(value: i32) -> i32 {
    value
}
fn hidden64(value: i64) -> i64 {
    value
}
/// Original left signed 32-bit operand.
pub fn left32(value: i32) -> i32 {
    hidden32(value)
}
/// Original right signed 32-bit operand.
pub fn right32(value: i32) -> i32 {
    hidden32(value)
}
/// Original left signed 64-bit operand.
pub fn left64(value: i64) -> i64 {
    hidden64(value)
}
/// Original right signed 64-bit operand.
pub fn right64(value: i64) -> i64 {
    hidden64(value)
}
