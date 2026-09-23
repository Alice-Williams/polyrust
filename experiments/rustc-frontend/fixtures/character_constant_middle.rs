//! Alias-only facade; no copied constant storage.
pub use constants::{
    AFTER_SURROGATES, ASCII, BEFORE_SURROGATES, COMPUTED, COPIED, MAXIMUM, NONCHARACTER,
    SAME_INTEGER, SUPPLEMENTARY, ZERO,
};
pub use second::{MAXIMUM as OTHER_MAXIMUM, SAME as OTHER_SAME};
/// Nested aliases retain their original owner.
pub mod nested {
    pub use super::MAXIMUM as AGAIN;
    pub use crate::nested as cycle;
}
