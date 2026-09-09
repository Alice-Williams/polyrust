//! C conversions must not inherit Rust's saturating float-to-integer behavior.
use super::super::CSafetyError as E;
use super::{CInteger, CNumber, numeric::limits};
use crate::ast::{CScalarRepresentation, CScalarType};

impl CNumber {
    pub(in crate::ownership) fn convert(self, ty: CScalarType) -> Result<Self, E> {
        if ty == CScalarType::F64 {
            return Ok(Self::Double(self.double()));
        }
        if ty == CScalarType::Bool {
            return Ok(Self::Integer(CInteger::checked(
                ty,
                i128::from(self.truth()),
            )?));
        }
        match self {
            Self::Integer(value) => Ok(Self::Integer(value.convert(ty)?)),
            Self::Double(value) => {
                let (minimum, maximum) = limits(ty)?;
                let truncated = value.trunc();
                // The exclusive upper power of two is exactly representable;
                // i64::MAX/u64::MAX are not exactly representable as f64.
                let exclusive = match ty.representation() {
                    CScalarRepresentation::Signed(_) | CScalarRepresentation::Unsigned(_) => {
                        (maximum + 1) as f64
                    }
                    _ => unreachable!("integer target after Bool/F64 branches"),
                };
                if !truncated.is_finite() || truncated < minimum as f64 || truncated >= exclusive {
                    return Err(E::IntegerRange);
                }
                Ok(Self::Integer(CInteger::checked(ty, truncated as i128)?))
            }
        }
    }
}
