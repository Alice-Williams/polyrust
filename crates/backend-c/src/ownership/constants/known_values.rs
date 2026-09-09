//! Values of closed constants on the pinned ABI; types come from their identity.
use super::super::CSafetyError as E;
use super::CInteger;
use crate::ast::{CKnownConstant as K, CObjectTypeKind};

pub(super) fn known(value: K) -> Result<CInteger, E> {
    let integer = match value {
        K::CharBit => 8,
        K::IntMin | K::I32Min => i128::from(i32::MIN),
        K::IntMax | K::I32Max => i128::from(i32::MAX),
        K::U32Max => i128::from(u32::MAX),
        K::I64Min => i128::from(i64::MIN),
        K::I64Max => i128::from(i64::MAX),
        K::U64Max | K::SizeMax => i128::from(u64::MAX),
        K::FloatRadix => 2,
        K::DoubleMantissaDigits => 53,
        K::DoubleMinExponent => -1021,
        K::DoubleMaxExponent => 1024,
        K::FloatEvaluationMethod => 0,
        K::EndOfFile => -1,
        K::StandardInput | K::StandardOutput | K::StandardError => {
            return Err(E::ExpectedNumericConstant);
        }
    };
    let ty = value.ty();
    let CObjectTypeKind::Scalar(ty) = ty.kind() else {
        unreachable!("integer known constant")
    };
    CInteger::checked(*ty, integer)
}
