#![no_std]
#![forbid(unsafe_code)]

//! Exact finite IEEE binary64 values for typed target literal nodes.
//!
//! This crate contains neither a target formatter nor floating arithmetic.
//! Equality and hashing preserve representation: positive and negative zero
//! are distinct. They must NOT implement source-language numeric comparison.
//!
//! ```
//! use portable_binary64::{Binary64Sign, FiniteBinary64, FiniteClass};
//! let value = FiniteBinary64::from_bits(0x8000_0000_0000_0000)?;
//! assert_eq!(value.sign(), Binary64Sign::Negative);
//! assert_eq!(value.class(), FiniteClass::Zero);
//! assert_eq!(value.parts().significand(), 0);
//! # Ok::<(), portable_binary64::NonFiniteBinary64>(())
//! ```

const FRACTION_MASK: u64 = (1_u64 << 52) - 1;
const EXPONENT_MASK: u64 = 0x7ff;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Binary64Sign {
    Positive,
    Negative,
}

impl Binary64Sign {
    const fn from_bits(bits: u64) -> Self {
        if bits >> 63 == 0 {
            Self::Positive
        } else {
            Self::Negative
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FiniteClass {
    Zero,
    Subnormal,
    Normal,
}

/// Rejection categories are not an alternative constructible finite payload.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NonFiniteBinary64 {
    Infinity(Binary64Sign),
    NaN,
}

impl core::fmt::Display for NonFiniteBinary64 {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str(match self {
            Self::Infinity(Binary64Sign::Positive) => "positive infinity is not a finite literal",
            Self::Infinity(Binary64Sign::Negative) => "negative infinity is not a finite literal",
            Self::NaN => "NaN is not a finite literal",
        })
    }
}
impl core::error::Error for NonFiniteBinary64 {}

/// A checked finite literal payload, not a general f64 value or target AST.
/// Safe callers cannot fabricate its representation.
///
/// ```compile_fail
/// use portable_binary64::FiniteBinary64;
/// let invalid = FiniteBinary64 { bits: 0x7ff0_0000_0000_0000 };
/// ```
/// Ordering, like equality, compares representations rather than numeric values.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FiniteBinary64 {
    bits: u64,
}

impl FiniteBinary64 {
    /// Validate unknown bits without floating arithmetic or canonicalization.
    pub const fn from_bits(bits: u64) -> Result<Self, NonFiniteBinary64> {
        if (bits >> 52) & EXPONENT_MASK == EXPONENT_MASK {
            if bits & FRACTION_MASK == 0 {
                Err(NonFiniteBinary64::Infinity(Binary64Sign::from_bits(bits)))
            } else {
                Err(NonFiniteBinary64::NaN)
            }
        } else {
            Ok(Self { bits })
        }
    }

    pub const fn to_bits(self) -> u64 {
        self.bits
    }

    pub const fn sign(self) -> Binary64Sign {
        Binary64Sign::from_bits(self.bits)
    }

    pub const fn class(self) -> FiniteClass {
        if (self.bits >> 52) & EXPONENT_MASK != 0 {
            FiniteClass::Normal
        } else if self.bits & FRACTION_MASK != 0 {
            FiniteClass::Subnormal
        } else {
            FiniteClass::Zero
        }
    }

    /// Exact sign * significand * 2^exponent, evaluated mathematically.
    /// Both zeros deliberately retain their sign with exponent zero.
    pub const fn parts(self) -> FiniteBinary64Parts {
        let fraction = self.bits & FRACTION_MASK;
        let (significand, exponent) = match self.class() {
            FiniteClass::Zero => (0, 0),
            FiniteClass::Subnormal => (fraction, -1074),
            FiniteClass::Normal => (
                (1_u64 << 52) | fraction,
                ((self.bits >> 52) & EXPONENT_MASK) as i16 - 1075,
            ),
        };
        FiniteBinary64Parts {
            sign: self.sign(),
            significand,
            exponent,
        }
    }
}

/// Decomposition can be obtained only from a checked finite value.
/// It contains numbers, never target source tokens.
///
/// ```compile_fail
/// use portable_binary64::{Binary64Sign, FiniteBinary64Parts};
/// let invalid = FiniteBinary64Parts {
///     sign: Binary64Sign::Positive, significand: u64::MAX, exponent: i16::MAX,
/// };
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FiniteBinary64Parts {
    sign: Binary64Sign,
    significand: u64,
    exponent: i16,
}

impl FiniteBinary64Parts {
    pub const fn sign(self) -> Binary64Sign {
        self.sign
    }

    /// At most 53 bits; zero only represents a signed zero.
    pub const fn significand(self) -> u64 {
        self.significand
    }

    /// Range -1074..=971, with zero using exponent zero.
    pub const fn exponent(self) -> i16 {
        self.exponent
    }
}

#[cfg(test)]
mod tests;
