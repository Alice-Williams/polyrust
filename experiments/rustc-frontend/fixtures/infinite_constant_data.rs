//! Original infinity constants: exact signs and compiler-evaluated expressions.
/// Positive infinity; changing its sign must invalidate every dependent.
pub const NAMED_POSITIVE: f64 = f64::INFINITY;
/// Negative infinity has a distinct constant representation.
pub const NAMED_NEGATIVE: f64 = f64::NEG_INFINITY;
pub const NEGATE_NEGATIVE: f64 = -f64::NEG_INFINITY;
pub const NEGATE_POSITIVE: f64 = -f64::INFINITY;
pub const POSITIVE_OVER_POSITIVE_ZERO: f64 = 1.0 / 0.0;
pub const POSITIVE_OVER_NEGATIVE_ZERO: f64 = 1.0 / -0.0;
pub const NEGATIVE_OVER_NEGATIVE_ZERO: f64 = -1.0 / -0.0;
pub const NEGATIVE_OVER_POSITIVE_ZERO: f64 = -1.0 / 0.0;
pub const POSITIVE_ADD_OVERFLOW: f64 = f64::MAX + f64::MAX;
pub const NEGATIVE_ADD_OVERFLOW: f64 = -f64::MAX - f64::MAX;
pub const POSITIVE_MUL_OVERFLOW: f64 = f64::MAX * 2.0;
pub const NEGATIVE_MUL_OVERFLOW: f64 = -f64::MAX * 2.0;
pub const POSITIVE_DIV_OVERFLOW: f64 = 1.0 / f64::from_bits(1);
pub const NEGATIVE_DIV_OVERFLOW: f64 = -1.0 / f64::from_bits(1);
pub const BITS_POSITIVE: f64 = f64::from_bits(0x7ff0_0000_0000_0000);
pub const BITS_NEGATIVE: f64 = f64::from_bits(0xfff0_0000_0000_0000);
pub const NEGATIVE_TIMES_NEGATIVE: f64 = f64::NEG_INFINITY * -2.0;
pub const POSITIVE_TIMES_NEGATIVE: f64 = f64::INFINITY * -2.0;
pub const ALIAS_POSITIVE: f64 = NAMED_POSITIVE;
pub const ALIAS_NEGATIVE: f64 = NAMED_NEGATIVE;
pub use NAMED_POSITIVE as PUBLIC_ALIAS;
