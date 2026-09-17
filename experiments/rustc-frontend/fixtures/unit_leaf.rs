//! Unit result producer with ordinary scalar arguments.
#![allow(clippy::unused_unit)]
/// Shared unit-fixture constant.
pub const MARK: i32 = 7;
/// Trace E.
pub fn empty() {}
/// Trace U.
pub fn explicit() -> () {
    ()
}
/// Trace A.
pub fn first(value: i32) -> i32 {
    value
}
/// Trace B.
pub fn second(value: i32) -> i32 {
    !value
}
/// Trace P.
pub fn predicate(flag: bool) -> bool {
    flag
}
/// Trace O.
pub fn observe(left: i32, right: i32, flag: bool) {
    let same = left == right;
    if flag {
        empty();
    } else {
        explicit();
    }
    if same {
        ()
    }
}
