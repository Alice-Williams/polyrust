//! Exact public constants, including nontrivial compile-time evaluation.
/// False constant documentation.
pub const FALSE: bool = false;
/// True constant documentation.
pub const TRUE: bool = true;
pub const I32_MIN: i32 = i32::MIN;
pub const I32_MAX: i32 = i32::MAX;
pub const I64_MIN: i64 = i64::MIN;
pub const I64_MAX: i64 = i64::MAX;
pub const FORWARD: i64 = WIDE;
pub const WIDE: i64 = 9_007_199_254_740_993;
pub const NEGATIVE_WIDE: i64 = -9_007_199_254_740_993;
pub const COMPUTED: i32 = 7 * 9 - 1;
pub use WIDE as WIDE_ALIAS;
