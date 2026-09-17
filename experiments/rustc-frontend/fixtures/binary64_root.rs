//! Public values cross three crates; private implementation remains private.
struct Pair {
    first: f64,
    second: f64,
}
fn keep(value: f64) -> f64 {
    value
}
/// Trace L.
fn left(value: f64) -> f64 {
    value
}
/// Trace R.
fn right(value: f64) -> f64 {
    value
}
pub fn identity(a: f64) -> f64 {
    a
}
pub fn local(a: f64) -> f64 {
    let value = keep(a);
    keep(value)
}
pub fn imported(a: f64) -> f64 {
    binary64_middle::relay(a)
}
pub fn shared(a: f64) -> f64 {
    let reference = &a;
    *reference
}
pub fn record(a: f64) -> f64 {
    let pair = Pair {
        first: keep(a),
        second: 0.0,
    };
    if pair.second == 0.0 { pair.first } else { 1.0 }
}
pub fn shared_record(a: f64) -> f64 {
    let pair = Pair {
        first: a,
        second: 0.0,
    };
    let reference = &pair;
    if reference.second == 0.0 {
        reference.first
    } else {
        1.0
    }
}
pub fn equal(a: f64, b: f64) -> bool {
    left(a) == right(b)
}
pub fn not_equal(a: f64, b: f64) -> bool {
    left(a) != right(b)
}
pub fn less(a: f64, b: f64) -> bool {
    left(a) < right(b)
}
pub fn less_equal(a: f64, b: f64) -> bool {
    left(a) <= right(b)
}
pub fn greater(a: f64, b: f64) -> bool {
    left(a) > right(b)
}
pub fn greater_equal(a: f64, b: f64) -> bool {
    left(a) >= right(b)
}
pub fn tenth() -> f64 {
    binary64_middle::tenth()
}
