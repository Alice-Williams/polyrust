//! Left consumer of the shared leaf.
pub fn score(value: i32) -> i32 {
    leaf::choose(value < 0, leaf::zero(), leaf::alias(value))
}

pub fn flip(value: bool) -> bool {
    leaf::flip(value)
}

pub fn zero() -> i32 {
    leaf::zero()
}
