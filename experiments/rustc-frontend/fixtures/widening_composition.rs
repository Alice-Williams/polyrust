//! Signed widening composes with ordinary records and existing scalar mappings.
#![forbid(unsafe_code)]
struct Record {
    value: i32,
}
fn input(value: i32) -> i32 {
    value
}
pub fn direct(value: i32) -> i64 {
    value as i64
}
pub fn literal(_value: i32) -> i64 {
    -2147483648i32 as i64
}
pub fn nested(value: i32) -> i64 {
    input(input(value)) as i64
}
pub fn borrowed(value: i32) -> i64 {
    let reference = &value;
    *reference as i64
}
pub fn record(value: i32) -> i64 {
    let record = Record { value };
    record.value as i64
}
pub fn composed(value: i32) -> i64 {
    (value.wrapping_mul(3) as i64).wrapping_add(1)
}
pub fn grouped(value: i32) -> i64 {
    (!(value as i64)).wrapping_sub(7)
}
