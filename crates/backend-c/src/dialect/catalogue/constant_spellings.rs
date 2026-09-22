//! Closed standard identities have one catalogue spelling, never caller text.
use crate::ast::CKnownConstant;

impl CKnownConstant {
    pub const fn spelling(self) -> &'static str {
        match self {
            Self::CharBit => "CHAR_BIT",
            Self::IntMin => "INT_MIN",
            Self::IntMax => "INT_MAX",
            Self::I32Min => "INT32_MIN",
            Self::I32Max => "INT32_MAX",
            Self::U32Max => "UINT32_MAX",
            Self::I64Min => "INT64_MIN",
            Self::I64Max => "INT64_MAX",
            Self::U64Max => "UINT64_MAX",
            Self::SizeMax => "SIZE_MAX",
            Self::FloatRadix => "FLT_RADIX",
            Self::DoubleMantissaDigits => "DBL_MANT_DIG",
            Self::DoubleMinExponent => "DBL_MIN_EXP",
            Self::DoubleMaxExponent => "DBL_MAX_EXP",
            Self::FloatEvaluationMethod => "FLT_EVAL_METHOD",
            Self::DoubleHasSubnormals => "DBL_HAS_SUBNORM",
            Self::DoubleInfinity => "HUGE_VAL",
            Self::EndOfFile => "EOF",
            Self::StandardInput => "stdin",
            Self::StandardOutput => "stdout",
            Self::StandardError => "stderr",
        }
    }
}
