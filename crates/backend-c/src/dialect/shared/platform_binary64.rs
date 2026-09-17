//! Closed binary64 platform properties, not user-provided assertion text.
use crate::ast::CKnownConstant;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Binary64Property {
    Radix,
    Mantissa,
    MinimumExponent,
    MaximumExponent,
    Subnormals,
    Evaluation,
}

impl Binary64Property {
    pub(super) const ALL: [Self; 6] = [
        Self::Radix,
        Self::Mantissa,
        Self::MinimumExponent,
        Self::MaximumExponent,
        Self::Subnormals,
        Self::Evaluation,
    ];

    pub(super) const fn constant(self) -> CKnownConstant {
        match self {
            Self::Radix => CKnownConstant::FloatRadix,
            Self::Mantissa => CKnownConstant::DoubleMantissaDigits,
            Self::MinimumExponent => CKnownConstant::DoubleMinExponent,
            Self::MaximumExponent => CKnownConstant::DoubleMaxExponent,
            Self::Subnormals => CKnownConstant::DoubleHasSubnormals,
            Self::Evaluation => CKnownConstant::FloatEvaluationMethod,
        }
    }

    pub(super) const fn expected(self) -> i32 {
        match self {
            Self::Radix => 2,
            Self::Mantissa => 53,
            Self::MinimumExponent => -1021,
            Self::MaximumExponent => 1024,
            Self::Subnormals => 1,
            Self::Evaluation => 0,
        }
    }
}
