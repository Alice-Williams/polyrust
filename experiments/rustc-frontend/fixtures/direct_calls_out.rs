//! An explicitly declared out-of-line source module.
pub(super) fn select(flag: bool, left: i32, right: i32) -> i32 {
    if flag { right } else { left }
}

pub(super) fn echo(value: i32) -> i32 {
    value
}
