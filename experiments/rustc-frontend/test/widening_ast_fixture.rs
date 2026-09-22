//! Closed typed-probe fixture, not a general Rust grammar.
pub fn input(value: i32) -> i32 {
    value
}
pub fn direct(value: i32) -> i64 {
    value as i64
}
pub fn call(value: i32) -> i64 {
    input(value) as i64
}
pub fn nested_call(value: i32) -> i64 {
    input(input(value)) as i64
}
pub fn literal() -> i64 {
    -2147483648i32 as i64
}
pub fn borrowed(value: i32) -> i64 {
    let borrow = &value;
    *borrow as i64
}
