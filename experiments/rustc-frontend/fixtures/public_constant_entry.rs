//! The selected-entry projection exports only score, not this public constant.
pub const VALUE: i32 = 62;

pub fn score(value: i32) -> i32 {
    if value > 0 { VALUE } else { 0 }
}
