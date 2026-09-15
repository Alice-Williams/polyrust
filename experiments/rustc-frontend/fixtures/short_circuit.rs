//! Lazy bool operators with operand calls, places and nested source contexts.
#![allow(clippy::nonminimal_bool)]

struct Flags {
    first: bool,
    second: bool,
}

/// Trace A.
fn first(value: bool) -> bool {
    value
}
/// Trace B.
fn second(value: bool) -> bool {
    value
}
/// Trace C.
fn third(value: bool) -> bool {
    value
}
fn equal(left: bool, right: bool) -> bool {
    left == right
}

pub fn and(a: bool, b: bool, _c: bool) -> bool {
    first(a) && second(b)
}
pub fn or(a: bool, b: bool, _c: bool) -> bool {
    first(a) || second(b)
}
pub fn nested(a: bool, b: bool, c: bool) -> bool {
    first(a) && (second(b) || third(c))
}
pub fn left_nested(a: bool, b: bool, c: bool) -> bool {
    (first(a) || second(b)) && third(c)
}
pub fn negated(a: bool, b: bool, c: bool) -> bool {
    !(first(a) && !second(b)) || third(c)
}
pub fn argument(a: bool, b: bool, c: bool) -> bool {
    equal(first(a) && second(b), third(c))
}
pub fn later_argument(a: bool, b: bool, c: bool) -> bool {
    equal(first(a), second(b) && third(c))
}
pub fn local(a: bool, b: bool, c: bool) -> bool {
    let value = first(a) && second(b);
    value || third(c)
}
pub fn shadow(a: bool, b: bool, c: bool) -> bool {
    let a = first(a) && second(b);
    let a = !a;
    a && third(c)
}
pub fn field(a: bool, b: bool, c: bool) -> bool {
    let flags = Flags {
        first: a,
        second: b,
    };
    flags.first && flags.second || c
}
pub fn shared(a: bool, b: bool, c: bool) -> bool {
    let reference = &a;
    (*reference && b) || c
}
pub fn comparison(a: bool, b: bool, c: bool) -> bool {
    (a == b) && (b != c)
}
pub fn conditional(a: bool, b: bool, c: bool) -> bool {
    if first(a) && second(b) {
        third(c)
    } else {
        false
    }
}
pub fn constants(a: bool, b: bool, c: bool) -> bool {
    let skipped = false && first(a);
    let taken = true || second(b);
    skipped || taken && third(c)
}
