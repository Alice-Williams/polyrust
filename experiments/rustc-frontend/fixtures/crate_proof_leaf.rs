//! Shared leaf crate documentation.
/// Private concrete storage stays inside the leaf implementation.
struct Cell {
    /// Private stored scalar.
    value: i32,
}

/// Private leaf helper.
fn hidden(value: i32) -> i32 {
    let cell = Cell { value };
    cell.value
}

/// Private ancestor of a publicly aliased operation.
mod internal {
    /// Public leaf score behind two aliases.
    pub fn score(value: i32) -> i32 {
        if value < 0 {
            super::hidden(0)
        } else {
            super::hidden(value)
        }
    }
}

pub use internal::score;
pub use internal::score as alias;

/// Leaf boolean operation.
pub fn flip(value: bool) -> bool {
    if value { boolean(false) } else { boolean(true) }
}

/// A private boolean helper exercises same-crate scalar calls in both branches.
fn boolean(value: bool) -> bool {
    value
}

/// Leaf zero-argument operation.
pub fn zero() -> i32 {
    42
}

/// Leaf mixed-parameter operation.
pub fn choose(flag: bool, left: i32, right: i32) -> i32 {
    if flag { left } else { right }
}
