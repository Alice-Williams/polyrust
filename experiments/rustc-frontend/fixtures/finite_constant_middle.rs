//! Alias-only façade; constants retain their original producer.
pub use constants::{
    FORWARD, HALF_INTEGER, HALF_MAX, HALF_NORMAL, HALF_ULP, INDEXED, MAX_FINITE, MAX_SUBNORMAL,
    MIN_NORMAL, MIN_SUBNORMAL, NEGATIVE_MAX, NEGATIVE_ZERO, ONE_ULP, POSITIVE_ZERO, SAME_VALUE,
    SUM, TENTH, THIRD, TWO_SUBNORMALS, ZERO_PRODUCT,
};
pub use second::SAME_VALUE as OTHER_SAME;
pub use second::TENTH as OTHER_TENTH;
/// Aliases preserve original module identity, including a finite module cycle.
pub mod nested {
    pub use super::TENTH as AGAIN;
    pub use crate::nested as cycle;
}
