#![forbid(unsafe_code)]
fn left32(value: i32) -> i32 {
    value
}
fn right32(value: i32) -> i32 {
    value
}
fn left64(value: i64) -> i64 {
    value
}
fn right64(value: i64) -> i64 {
    value
}
pub fn method32(a: i32, b: i32) -> i32 {
    left32(a).wrapping_add(right32(b))
}
pub fn associated32(a: i32, b: i32) -> i32 {
    i32::wrapping_add(left32(a), right32(b))
}
pub fn nested32(a: i32, b: i32) -> i32 {
    a.wrapping_add(b).wrapping_add(1)
}
pub fn grouped32(a: i32, b: i32) -> i32 {
    !(a.wrapping_add(b))
}
pub fn method64(a: i64, b: i64) -> i64 {
    left64(a).wrapping_add(right64(b))
}
pub fn associated64(a: i64, b: i64) -> i64 {
    i64::wrapping_add(left64(a), right64(b))
}
pub fn nested64(a: i64, b: i64) -> i64 {
    a.wrapping_add(b).wrapping_add(1)
}
pub fn grouped64(a: i64, b: i64) -> i64 {
    !(a.wrapping_add(b))
}
fn wrapping_add(left: i32, _right: i32) -> i32 {
    left
}
pub fn ordinary_name(a: i32, b: i32) -> i32 {
    wrapping_add(a, b)
}
