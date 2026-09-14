//! Root crate diamond documentation.
use left as again;

/// Nested foreign calls retain independent owning implementations.
pub fn score(value: i32) -> i32 {
    left::score(right::score(value))
}
pub use score as alias;

pub fn alternate(value: i32) -> i32 {
    again::score(value)
}

pub fn flip(value: bool) -> bool {
    left::flip(right::flip(value))
}

pub fn zero() -> i32 {
    left::zero()
}

pub fn choose(flag: bool, first: i32, second: i32) -> i32 {
    if flag {
        left::score(first)
    } else {
        right::score(second)
    }
}
