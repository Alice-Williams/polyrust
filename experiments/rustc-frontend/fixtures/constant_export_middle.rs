//! Alias-only crate: café λ — no source function or constant definitions.
pub use constants::{
    COMPUTED, FALSE, FORWARD, I32_MAX, I32_MIN, I64_MAX, I64_MIN, NEGATIVE_WIDE, TRUE, WIDE,
};
pub use second::EXTRA;
/// Finite local module-alias cycle: façade 日本語.
pub mod nested {
    pub use super::COMPUTED as AGAIN;
    pub use crate::nested as cycle;
}
