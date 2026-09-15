//! Source-owned wide values; no arithmetic or helper runtime is required.
pub use identity as exported_identity;

struct Pair {
    left: i64,
    right: i64,
    select: bool,
}
fn keep(value: i64) -> i64 {
    value
}
/// Trace L.
fn left(value: i64) -> i64 {
    value
}
/// Trace R.
fn right(value: i64) -> i64 {
    value
}
/// Trace C.
fn witness(value: bool) -> bool {
    value
}

pub fn identity(a: i64, _b: i64, _flag: bool) -> i64 {
    a
}
pub fn choose(a: i64, b: i64, flag: bool) -> i64 {
    if (left(a) < right(b)) == flag { b } else { a }
}
pub fn minimum(_a: i64, _b: i64, _flag: bool) -> i64 {
    -9_223_372_036_854_775_808i64
}
pub fn maximum(_a: i64, _b: i64, _flag: bool) -> i64 {
    9_223_372_036_854_775_807
}
pub fn positive(_a: i64, _b: i64, _flag: bool) -> i64 {
    9_007_199_254_740_993i64
}
pub fn negative(_a: i64, _b: i64, _flag: bool) -> i64 {
    -9_007_199_254_740_993
}
pub fn local(a: i64, _b: i64, _flag: bool) -> i64 {
    let value = keep(a);
    let value = keep(value);
    keep(value)
}
pub fn record(a: i64, b: i64, flag: bool) -> i64 {
    let pair = Pair {
        left: keep(a),
        right: keep(b),
        select: flag,
    };
    if pair.select { pair.right } else { pair.left }
}
pub fn shared(a: i64, _b: i64, _flag: bool) -> i64 {
    let reference = &a;
    *reference
}
pub fn shared_record(a: i64, b: i64, flag: bool) -> i64 {
    let pair = Pair {
        left: a,
        right: b,
        select: flag,
    };
    let reference = &pair;
    if reference.select {
        reference.right
    } else {
        reference.left
    }
}
pub fn imported(a: i64, b: i64, flag: bool) -> i64 {
    i64_leaf::choose(
        i64_leaf::identity(a),
        -2_147_483_648,
        i64_leaf::identity(b),
        flag,
    )
}
pub fn mixed(a: i64, b: i64, flag: bool) -> i64 {
    if (i64_leaf::narrow(-2_147_483_648) < 0) && (flag || i64_leaf::equal(a, b)) {
        a
    } else {
        b
    }
}
pub fn equal(a: i64, b: i64, _flag: bool) -> bool {
    witness(left(a) == right(b))
}
pub fn not_equal(a: i64, b: i64, _flag: bool) -> bool {
    witness(left(a) != right(b))
}
pub fn less(a: i64, b: i64, _flag: bool) -> bool {
    witness(left(a) < right(b))
}
pub fn less_equal(a: i64, b: i64, _flag: bool) -> bool {
    witness(left(a) <= right(b))
}
pub fn greater(a: i64, b: i64, _flag: bool) -> bool {
    witness(left(a) > right(b))
}
pub fn greater_equal(a: i64, b: i64, _flag: bool) -> bool {
    witness(left(a) >= right(b))
}
pub fn lazy(a: i64, b: i64, flag: bool) -> bool {
    ((left(a) < right(b)) && flag) || i64_leaf::equal(left(a), right(b))
}
