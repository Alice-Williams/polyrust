//! Typed wrapping-negation source proof; no special generator annotations.
/// Trace W32.
fn observe32(value: i32) -> i32 {
    wrapping_leaf::identity32(value)
}
mod narrow {
    pub fn wrapping_neg(value: i32) -> i32 {
        value
    }
}
pub fn method32(value: i32) -> i32 {
    observe32(value).wrapping_neg()
}
pub fn associated32(value: i32) -> i32 {
    i32::wrapping_neg(observe32(value))
}
pub fn nested32(value: i32) -> i32 {
    observe32(value).wrapping_neg().wrapping_neg()
}
pub fn leaf32(value: i32) -> i32 {
    wrapping_leaf::negate32(observe32(value))
}
pub fn ordinary32(value: i32) -> i32 {
    narrow::wrapping_neg(observe32(value))
}
pub fn absolute32(value: i32) -> i32 {
    if value < 0 {
        observe32(value).wrapping_neg()
    } else {
        observe32(value)
    }
}
/// Trace W64.
fn observe64(value: i64) -> i64 {
    wrapping_leaf::identity64(value)
}
mod wide {
    pub fn wrapping_neg(value: i64) -> i64 {
        value
    }
}
pub fn method64(value: i64) -> i64 {
    observe64(value).wrapping_neg()
}
pub fn associated64(value: i64) -> i64 {
    i64::wrapping_neg(observe64(value))
}
pub fn nested64(value: i64) -> i64 {
    observe64(value).wrapping_neg().wrapping_neg()
}
pub fn leaf64(value: i64) -> i64 {
    wrapping_leaf::negate64(observe64(value))
}
pub fn ordinary64(value: i64) -> i64 {
    wide::wrapping_neg(observe64(value))
}
pub fn absolute64(value: i64) -> i64 {
    if value < 0 {
        observe64(value).wrapping_neg()
    } else {
        observe64(value)
    }
}
