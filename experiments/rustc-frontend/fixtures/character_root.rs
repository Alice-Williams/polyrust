//! Public root forwarding with no copied producer implementation.
#![forbid(unsafe_code)]
use character_middle::operations as original;
pub use identity as scalar_alias;

pub fn identity(value: char) -> char {
    original::forward(value)
}
pub fn local(value: char) -> char {
    original::local(value)
}
pub fn select(condition: bool, left: char, right: char) -> char {
    original::select(condition, left, right)
}
pub fn record_character(value: char, marker: i32) -> char {
    original::record_character(value, marker)
}
pub fn record_marker(value: char, marker: i32) -> i32 {
    original::record_marker(value, marker)
}
pub fn nested_less(left: char, right: char) -> bool {
    original::nested_less(left, right)
}
