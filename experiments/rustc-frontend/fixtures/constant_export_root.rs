//! Mixed source owner: naïve Δ — transitive aliases and one real reader.
pub use middle::COMPUTED as RENAMED;
pub use middle::{
    COMPUTED, EXTRA, FALSE, FORWARD, I32_MAX, I32_MIN, I64_MAX, I64_MIN, NEGATIVE_WIDE, TRUE, WIDE,
};
pub const OWN: i32 = 42;
pub fn read() -> i32 {
    middle::COMPUTED
}
