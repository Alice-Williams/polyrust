//! Inclusive bounds always fit their actual target scalar representation.
use crate::ast::CScalarType;
use crate::ownership::{
    CSafetyError as E,
    constants::{CInteger, limits},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ownership) struct IntegerRange {
    ty: CScalarType,
    min: i128,
    max: i128,
}
impl IntegerRange {
    pub(super) fn checked(ty: CScalarType, min: i128, max: i128) -> Result<Self, E> {
        if min > max {
            return Err(E::InvalidNumericRange);
        }
        CInteger::checked(ty, min)?;
        CInteger::checked(ty, max)?;
        Ok(Self { ty, min, max })
    }
    pub(super) fn full(ty: CScalarType) -> Result<Self, E> {
        let (min, max) = limits(ty)?;
        Self::checked(ty, min, max)
    }
    pub(super) fn exact(value: CInteger) -> Self {
        Self {
            ty: value.ty(),
            min: value.value(),
            max: value.value(),
        }
    }
    pub(super) const fn ty(self) -> CScalarType {
        self.ty
    }
    pub(super) const fn min(self) -> i128 {
        self.min
    }
    pub(super) const fn max(self) -> i128 {
        self.max
    }
    pub(super) fn exact_value(self) -> Option<CInteger> {
        (self.min == self.max)
            .then(|| CInteger::checked(self.ty, self.min).expect("checked interval endpoint"))
    }
    pub(super) fn join(self, right: Self) -> Result<Self, E> {
        if self.ty != right.ty {
            return Err(E::InvalidNumericRange);
        }
        Self::checked(self.ty, self.min.min(right.min), self.max.max(right.max))
    }
    pub(super) const fn contains(self, value: i128) -> bool {
        self.min <= value && value <= self.max
    }
    pub(super) fn truth(self) -> Option<bool> {
        if self.min == 0 && self.max == 0 {
            Some(false)
        } else if !self.contains(0) {
            Some(true)
        } else {
            None
        }
    }
}
