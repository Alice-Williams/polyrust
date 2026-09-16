//! Public constant and function APIs share one checked crate identity.
pub fn read_forward() -> i64 {
    FORWARD
}
#[path = "public_constant_data.rs"]
mod data;
pub use data::*;
mod hidden {
    pub const COMPUTED: i32 = 17;
}
pub fn read_false() -> bool {
    FALSE
}
pub fn read_true() -> bool {
    TRUE
}
pub fn read_i32_min() -> i32 {
    I32_MIN
}
pub fn read_i32_max() -> i32 {
    I32_MAX
}
pub fn read_i64_min() -> i64 {
    I64_MIN
}
pub fn read_i64_max() -> i64 {
    I64_MAX
}
pub fn read_wide() -> i64 {
    WIDE_ALIAS
}
pub fn read_negative_wide() -> i64 {
    NEGATIVE_WIDE
}
pub fn read_computed() -> i32 {
    COMPUTED
}
pub fn read_private() -> i32 {
    hidden::COMPUTED
}
