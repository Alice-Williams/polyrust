//! Signed and floating narrowing require bounds before any conversion occurs.
use super::{CScalarRange as Range, CTransfer, integer::IntegerRange};
use crate::ast::{CScalarRepresentation as R, CScalarType};
use crate::ownership::{
    CSafetyError as E,
    constants::{CInteger, CNumber},
};

impl Range {
    pub(in crate::ownership) fn convert(self, ty: CScalarType) -> Result<CTransfer, E> {
        if ty == CScalarType::Bool {
            let range = match self.truth() {
                Some(value) => IntegerRange::checked(ty, i128::from(value), i128::from(value))?,
                None => IntegerRange::full(ty)?,
            };
            return Ok(CTransfer::NonWrapping(Self::Integer(range)));
        }
        let result = match self {
            Self::Integer(value) if ty == CScalarType::F64 => {
                let left =
                    CNumber::Integer(CInteger::checked(value.ty(), value.min())?).convert(ty)?;
                let right =
                    CNumber::Integer(CInteger::checked(value.ty(), value.max())?).convert(ty)?;
                CTransfer::NonWrapping(Self::exact(left).join(Self::exact(right))?)
            }
            Self::Integer(value) => integer(value, ty)?,
            Self::Float(value) if ty == CScalarType::F64 => {
                CTransfer::NonWrapping(Self::Float(value))
            }
            Self::Float(value) => {
                if value.may_nan() {
                    return Err(E::IntegerRange);
                }
                let (min, max) = value.bounds().ok_or(E::IntegerRange)?;
                let min = CNumber::Double(min).convert(ty)?.integer()?.value();
                let max = CNumber::Double(max).convert(ty)?.integer()?.value();
                CTransfer::NonWrapping(Self::Integer(IntegerRange::checked(ty, min, max)?))
            }
        };
        // Exact observations keep signed zero and narrow modulo results.
        if let Some(value) = self.exact_value() {
            Ok(result.with_range(Self::exact(value.convert(ty)?)))
        } else {
            Ok(result)
        }
    }
}

fn integer(value: IntegerRange, ty: CScalarType) -> Result<CTransfer, E> {
    match ty.representation() {
        R::Signed(_) => Ok(CTransfer::NonWrapping(Range::Integer(
            IntegerRange::checked(ty, value.min(), value.max())?,
        ))),
        R::Unsigned(width) => {
            let modulus = 1_i128 << width.bits();
            let interval = if value.min().div_euclid(modulus) == value.max().div_euclid(modulus) {
                IntegerRange::checked(
                    ty,
                    value.min().rem_euclid(modulus),
                    value.max().rem_euclid(modulus),
                )?
            } else {
                IntegerRange::full(ty)?
            };
            let interval = Range::Integer(interval);
            Ok(if value.min() >= 0 && value.max() < modulus {
                CTransfer::NonWrapping(interval)
            } else {
                CTransfer::MayWrap(interval)
            })
        }
        R::Bool | R::Binary64 => unreachable!("handled before integer conversion"),
    }
}
