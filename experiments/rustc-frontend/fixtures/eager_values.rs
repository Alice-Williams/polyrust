//! Eager operators compose with lazy control and real imported calls.
struct Flags {
    first: bool,
    second: bool,
    third: bool,
}
/// Trace A.
fn left(value: bool) -> bool {
    value
}
/// Trace B.
fn right(value: bool) -> bool {
    value
}
/// Trace C.
fn third(value: bool) -> bool {
    value
}

pub fn and(a: bool, b: bool, _c: bool) -> bool {
    left(a) & right(b)
}
pub fn or(a: bool, b: bool, _c: bool) -> bool {
    left(a) | right(b)
}
pub fn xor(a: bool, b: bool, _c: bool) -> bool {
    left(a) ^ right(b)
}
pub fn nested(a: bool, b: bool, c: bool) -> bool {
    !(left(a) & right(b)) ^ third(c)
}
pub fn lazy_left(a: bool, b: bool, c: bool) -> bool {
    (left(a) && right(b)) | third(c)
}
pub fn lazy_right(a: bool, b: bool, c: bool) -> bool {
    left(a) & (right(b) || third(c))
}
pub fn lazy_outer(a: bool, b: bool, c: bool) -> bool {
    left(a) && (right(b) ^ third(c))
}
pub fn eager_outer(a: bool, b: bool, c: bool) -> bool {
    (left(a) | right(b)) && third(c)
}
pub fn field(a: bool, b: bool, c: bool) -> bool {
    let flags = Flags {
        first: a,
        second: b,
        third: c,
    };
    if flags.first {
        flags.second ^ flags.third
    } else {
        flags.second
    }
}
pub fn shared(a: bool, b: bool, _c: bool) -> bool {
    let first = &a;
    let second = &b;
    !*first & *second
}
pub fn imported(a: bool, b: bool, c: bool) -> bool {
    eager_leaf::both(left(a), right(b)) ^ third(c)
}
pub fn call_args(a: bool, b: bool, c: bool) -> bool {
    eager_leaf::both(left(a) & right(b), third(c))
}
pub fn condition(a: bool, b: bool, c: bool) -> bool {
    if left(a) ^ right(b) {
        third(c)
    } else {
        !third(c)
    }
}
