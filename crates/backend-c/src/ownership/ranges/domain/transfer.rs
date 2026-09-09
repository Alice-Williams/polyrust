//! Every retained operand interval participates; one unsafe pair rejects proof.
use super::{E, NumericDomain};
use crate::{
    ast::{CBinaryOperator, CScalarType, CUnaryOperator},
    ownership::ranges::CTransfer,
};

#[derive(Clone, Debug, PartialEq)]
pub(in crate::ownership) struct DomainTransfer {
    pub(in crate::ownership) domain: NumericDomain,
    pub(in crate::ownership) loss: NumericLoss,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ownership) enum NumericLoss {
    None,
    MayWrap,
}
impl NumericLoss {
    pub(in crate::ownership) fn join(self, other: Self) -> Self {
        if self == Self::MayWrap || other == Self::MayWrap {
            Self::MayWrap
        } else {
            Self::None
        }
    }
}
impl DomainTransfer {
    fn new(ty: CScalarType) -> Self {
        Self {
            domain: NumericDomain::empty(ty),
            loss: NumericLoss::None,
        }
    }
    fn include(&mut self, value: CTransfer) -> Result<(), E> {
        self.domain = self.domain.join(&NumericDomain::range(value.range()))?;
        if matches!(value, CTransfer::MayWrap(_)) {
            self.loss = NumericLoss::MayWrap;
        }
        Ok(())
    }
}
impl NumericDomain {
    pub(in crate::ownership) fn convert(&self, ty: CScalarType) -> Result<DomainTransfer, E> {
        let mut result = DomainTransfer::new(ty);
        for part in self.parts() {
            result.include(part.convert(ty)?)?;
        }
        Ok(result)
    }
    pub(in crate::ownership) fn unary(
        &self,
        operator: CUnaryOperator,
    ) -> Result<DomainTransfer, E> {
        let mut result = DomainTransfer::new(operator.result_type(self.ty())?);
        for part in self.parts() {
            result.include(part.unary(operator)?)?;
        }
        Ok(result)
    }
    pub(in crate::ownership) fn binary(
        &self,
        operator: CBinaryOperator,
        right: &Self,
    ) -> Result<DomainTransfer, E> {
        let mut result = DomainTransfer::new(operator.result_type(self.ty(), right.ty())?);
        if !matches!(
            operator,
            CBinaryOperator::LogicalAnd
                | CBinaryOperator::LogicalOr
                | CBinaryOperator::ShiftLeft
                | CBinaryOperator::ShiftRight
        ) {
            let common = self.ty().usual_arithmetic_conversion(right.ty());
            result.loss = self.convert(common)?.loss.join(right.convert(common)?.loss);
        }
        for left in self.parts() {
            for right in right.parts() {
                result.include(left.binary(operator, right)?)?;
            }
        }
        Ok(result)
    }
}
