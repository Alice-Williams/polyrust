//! Compiler-resolved foreign constants and call/value dependency diamonds.
use constants::WIDE_ALIAS as RENAMED;
/// Consumer-owned constant, not an imported declaration.
pub const LOCAL: i32 = 29;
mod hidden {
    pub const COMPUTED: i32 = 17;
}
pub fn read_false() -> bool {
    constants::FALSE
}
pub fn read_true() -> bool {
    constants::TRUE
}
pub fn read_i32_min() -> i32 {
    constants::I32_MIN
}
pub fn read_i32_max() -> i32 {
    constants::I32_MAX
}
pub fn read_i64_min() -> i64 {
    constants::I64_MIN
}
pub fn read_i64_max() -> i64 {
    constants::I64_MAX
}
pub fn read_wide() -> i64 {
    RENAMED
}
pub fn read_negative_wide() -> i64 {
    constants::NEGATIVE_WIDE
}
pub fn read_computed() -> i32 {
    constants::COMPUTED
}
pub fn read_forward() -> i64 {
    constants::FORWARD
}
pub fn read_private() -> i32 {
    hidden::COMPUTED
}
pub fn read_local() -> i32 {
    LOCAL
}
pub fn read_left() -> i64 {
    left::via_dependency()
}
pub fn read_right() -> i64 {
    right::via_dependency()
}
