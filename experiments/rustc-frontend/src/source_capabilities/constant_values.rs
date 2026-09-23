//! Closed compiler-evaluated constant domain, independent of source literals.
//! Adding a literal kind cannot expand any constant capability implicitly.
use portable_binary64::{Binary64Sign, FiniteBinary64};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ScalarConstantValue {
    I32(i32),
    I64(i64),
    Bool(bool),
    F64(FiniteBinary64),
    Infinity(Binary64Sign),
    Char(char),
}

impl ScalarConstantValue {
    pub(crate) fn original(self) -> portable_codegen::RustConstantValue {
        use portable_codegen::RustConstantValue as Original;
        match self {
            Self::Bool(value) => Original::Bool(value),
            Self::I32(value) => Original::I32(value),
            Self::I64(value) => Original::I64(value),
            Self::F64(value) => Original::F64Bits(value.to_bits()),
            Self::Infinity(Binary64Sign::Positive) => Original::F64Bits(0x7ff0_0000_0000_0000),
            Self::Infinity(Binary64Sign::Negative) => Original::F64Bits(0xfff0_0000_0000_0000),
            Self::Char(value) => Original::Char(value),
        }
    }
}
