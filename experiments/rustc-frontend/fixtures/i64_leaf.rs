//! Independent wide-integer dependency, with exact exported signatures.
pub fn identity(value: i64) -> i64 {
    value
}
pub fn choose(a: i64, narrow: i32, b: i64, flag: bool) -> i64 {
    if flag && narrow < 0 { b } else { a }
}
pub fn equal(a: i64, b: i64) -> bool {
    a == b
}
pub fn narrow(value: i32) -> i32 {
    value
}
