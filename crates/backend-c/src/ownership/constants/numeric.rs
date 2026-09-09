//! Private constructors distinguish target values from arbitrary host numbers.
use super::super::CSafetyError as E;
use crate::ast::{CScalarRepresentation as R, CScalarType};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ownership) struct CInteger {
    ty: CScalarType,
    value: i128,
}

impl CInteger {
    pub(in crate::ownership) fn checked(ty: CScalarType, value: i128) -> Result<Self, E> {
        let (min, max) = limits(ty)?;
        if value < min || value > max {
            return Err(E::IntegerRange);
        }
        Ok(Self { ty, value })
    }
    pub(in crate::ownership) const fn ty(self) -> CScalarType {
        self.ty
    }
    pub(in crate::ownership) const fn value(self) -> i128 {
        self.value
    }
    pub(super) fn convert(self, ty: CScalarType) -> Result<Self, E> {
        let value = match ty.representation() {
            R::Bool => i128::from(self.value != 0),
            R::Unsigned(width) => self.value.rem_euclid(1_i128 << width.bits()),
            R::Signed(_) => self.value,
            R::Binary64 => return Err(E::ExpectedIntegerConstant),
        };
        Self::checked(ty, value)
    }
}

pub(super) fn limits(ty: CScalarType) -> Result<(i128, i128), E> {
    Ok(match ty.representation() {
        R::Bool => (0, 1),
        R::Signed(width) => {
            let bound = 1_i128 << (width.bits() - 1);
            (-bound, bound - 1)
        }
        R::Unsigned(width) => (0, (1_i128 << width.bits()) - 1),
        R::Binary64 => return Err(E::ExpectedIntegerConstant),
    })
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ownership) enum CNumber {
    Integer(CInteger),
    Double(f64),
}

impl CNumber {
    pub(in crate::ownership) fn truth(self) -> bool {
        match self {
            Self::Integer(value) => value.value() != 0,
            Self::Double(value) => value != 0.0,
        }
    }
    pub(super) fn ty(self) -> CScalarType {
        match self {
            Self::Integer(value) => value.ty(),
            Self::Double(_) => CScalarType::F64,
        }
    }
    pub(in crate::ownership) fn integer(self) -> Result<CInteger, E> {
        match self {
            Self::Integer(value) => Ok(value),
            Self::Double(_) => Err(E::ExpectedIntegerConstant),
        }
    }
    pub(super) fn double(self) -> f64 {
        match self {
            Self::Integer(value) => value.value() as f64,
            Self::Double(value) => value,
        }
    }
}
