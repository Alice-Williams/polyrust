//! Original finite constants: exact bits and compiler-evaluated expressions.
/// Positive zero; changing its sign must invalidate every dependent.
pub const POSITIVE_ZERO: f64 = 0.0;
/// Negative zero is a distinct constant representation.
pub const NEGATIVE_ZERO: f64 = -0.0;
pub const MIN_SUBNORMAL: f64 = f64::from_bits(1);
pub const MAX_SUBNORMAL: f64 = f64::from_bits(0x000f_ffff_ffff_ffff);
pub const MIN_NORMAL: f64 = f64::MIN_POSITIVE;
pub const MAX_FINITE: f64 = f64::MAX;
pub const NEGATIVE_MAX: f64 = -f64::MAX;
pub const ONE_ULP: f64 = f64::from_bits(0x3ff0_0000_0000_0001);
pub const TENTH: f64 = 0.1;
pub const HALF_INTEGER: f64 = 9_007_199_254_740_993.0;
pub const HALF_ULP: f64 = 1.0 + f64::from_bits(0x3ca0_0000_0000_0000);
pub const SUM: f64 = 0.1 + 0.2;
pub const THIRD: f64 = 1.0 / 3.0;
pub const HALF_NORMAL: f64 = f64::MIN_POSITIVE / 2.0;
pub const TWO_SUBNORMALS: f64 = f64::from_bits(1) * 2.0;
pub const HALF_MAX: f64 = f64::MAX / 2.0;
pub const ZERO_PRODUCT: f64 = -0.0 * 2.0;
/// Constant evaluation may use arrays without admitting runtime array indexing.
pub const INDEXED: f64 = [0.25, 0.5][1];
pub const FORWARD: f64 = TENTH;
pub const SAME_VALUE: f64 = 0.1;
pub use TENTH as TENTH_ALIAS;
