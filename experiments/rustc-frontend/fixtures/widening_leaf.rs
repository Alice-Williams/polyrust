//! Original signed operand and private implementation.
#![forbid(unsafe_code)]
fn hidden(value: i32) -> i32 {
    value
}
/// Original signed 32-bit operand.
pub fn input(value: i32) -> i32 {
    hidden(value)
}
