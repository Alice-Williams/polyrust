//! Built-in signed bitwise operations through ordinary Rust source.
struct Pair32 {
    first: i32,
    second: i32,
}
/// Trace L32.
fn left32(v: i32) -> i32 {
    v
}
/// Trace R32.
fn right32(v: i32) -> i32 {
    v
}
pub fn invert32(a: i32, _b: i32) -> i32 {
    !left32(a)
}
pub fn and32(a: i32, b: i32) -> i32 {
    left32(a) & right32(b)
}
pub fn or32(a: i32, b: i32) -> i32 {
    left32(a) | right32(b)
}
pub fn xor32(a: i32, b: i32) -> i32 {
    left32(a) ^ right32(b)
}
pub fn nested32(a: i32, b: i32) -> i32 {
    !(left32(a) & right32(b)) ^ (a | b)
}
pub fn field32(a: i32, b: i32) -> i32 {
    {
        let pair = Pair32 {
            first: a,
            second: b,
        };
        !pair.first ^ pair.second
    }
}
pub fn shared32(a: i32, b: i32) -> i32 {
    {
        let first = &a;
        let second = &b;
        !*first & *second
    }
}
pub fn imported32(a: i32, b: i32) -> i32 {
    bitwise_leaf::invert32(a) | b
}
struct Pair64 {
    first: i64,
    second: i64,
}
/// Trace L64.
fn left64(v: i64) -> i64 {
    v
}
/// Trace R64.
fn right64(v: i64) -> i64 {
    v
}
pub fn invert64(a: i64, _b: i64) -> i64 {
    !left64(a)
}
pub fn and64(a: i64, b: i64) -> i64 {
    left64(a) & right64(b)
}
pub fn or64(a: i64, b: i64) -> i64 {
    left64(a) | right64(b)
}
pub fn xor64(a: i64, b: i64) -> i64 {
    left64(a) ^ right64(b)
}
pub fn nested64(a: i64, b: i64) -> i64 {
    !(left64(a) & right64(b)) ^ (a | b)
}
pub fn field64(a: i64, b: i64) -> i64 {
    {
        let pair = Pair64 {
            first: a,
            second: b,
        };
        !pair.first ^ pair.second
    }
}
pub fn shared64(a: i64, b: i64) -> i64 {
    {
        let first = &a;
        let second = &b;
        !*first & *second
    }
}
pub fn imported64(a: i64, b: i64) -> i64 {
    bitwise_leaf::invert64(a) | b
}
