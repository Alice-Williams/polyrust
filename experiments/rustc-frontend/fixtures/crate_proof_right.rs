//! Right consumer of the same shared leaf.
pub fn score(value: i32) -> i32 {
    leaf::choose(value > 0, value, leaf::choose(value == 0, -1, leaf::zero()))
}

pub fn flip(value: bool) -> bool {
    leaf::flip(leaf::flip(value))
}
