//! Closed standard constants. Header/spelling/effect catalogues follow in 03.

use super::{CKnownObject, CObjectType, CPointerTarget, CScalarType};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CKnownConstant {
    CharBit,
    IntMin,
    IntMax,
    I32Min,
    I32Max,
    U32Max,
    I64Min,
    I64Max,
    U64Max,
    SizeMax,
    FloatRadix,
    DoubleMantissaDigits,
    DoubleMinExponent,
    DoubleMaxExponent,
    FloatEvaluationMethod,
    EndOfFile,
    StandardInput,
    StandardOutput,
    StandardError,
}

impl CKnownConstant {
    pub fn ty(self) -> CObjectType {
        let scalar = match self {
            Self::CharBit
            | Self::IntMin
            | Self::IntMax
            | Self::I32Min
            | Self::I32Max
            | Self::FloatRadix
            | Self::DoubleMantissaDigits
            | Self::DoubleMinExponent
            | Self::DoubleMaxExponent
            | Self::FloatEvaluationMethod
            | Self::EndOfFile => CScalarType::Int,
            Self::U32Max => CScalarType::U32,
            Self::I64Min | Self::I64Max => CScalarType::I64,
            Self::U64Max => CScalarType::U64,
            Self::SizeMax => CScalarType::Size,
            Self::StandardInput | Self::StandardOutput | Self::StandardError => {
                return CObjectType::pointer(CPointerTarget::Object(Box::new(CObjectType::known(
                    CKnownObject::File,
                ))));
            }
        };
        CObjectType::scalar(scalar)
    }

    pub const fn is_integer_constant_expression(self) -> bool {
        match self {
            Self::StandardInput | Self::StandardOutput | Self::StandardError => false,
            Self::CharBit
            | Self::IntMin
            | Self::IntMax
            | Self::I32Min
            | Self::I32Max
            | Self::U32Max
            | Self::I64Min
            | Self::I64Max
            | Self::U64Max
            | Self::SizeMax
            | Self::FloatRadix
            | Self::DoubleMantissaDigits
            | Self::DoubleMinExponent
            | Self::DoubleMaxExponent
            | Self::FloatEvaluationMethod
            | Self::EndOfFile => true,
        }
    }
}
