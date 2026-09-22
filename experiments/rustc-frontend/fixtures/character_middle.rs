//! Original middle owner; aliases retain producer declaration identity.
#![forbid(unsafe_code)]
pub use operations::forward as scalar_alias;

/// Named import resolution must retain the original producer certificate.
pub mod operations {
    use character_leaf::scalar_alias as original;

    pub fn forward(value: char) -> char {
        original(value)
    }
    pub fn local(value: char) -> char {
        character_leaf::local(value)
    }
    pub fn select(condition: bool, left: char, right: char) -> char {
        character_leaf::select(condition, original(left), original(right))
    }
    pub fn record_character(value: char, marker: i32) -> char {
        character_leaf::record_character(original(value), marker)
    }
    pub fn record_marker(value: char, marker: i32) -> i32 {
        character_leaf::record_marker(original(value), marker)
    }
    pub fn nested_less(left: char, right: char) -> bool {
        character_leaf::compare::less(original(left), original(right))
    }
}
