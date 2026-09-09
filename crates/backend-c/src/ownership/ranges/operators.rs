//! Exact and interval inputs share the same promoted operation relation.
use super::{
    CScalarRange as Range, CTransfer, floating::FloatRange, integer::IntegerRange,
    integer_operations, shifts,
};
use crate::ast::{
    CBinaryOperator as B, CScalarRepresentation as R, CScalarType, CUnaryOperator as U,
};
use crate::ownership::{CSafetyError as E, constants::limits};

pub(super) fn boolean(value: Option<bool>) -> Result<CTransfer, E> {
    let (min, max) = value.map_or((0, 1), |value| (i128::from(value), i128::from(value)));
    Ok(CTransfer::NonWrapping(Range::Integer(
        IntegerRange::checked(CScalarType::Int, min, max)?,
    )))
}

impl Range {
    pub(in crate::ownership) fn binary(self, op: B, right: Self) -> Result<CTransfer, E> {
        let result_type = op.result_type(self.ty(), right.ty())?;
        let exact = self
            .exact_value()
            .zip(right.exact_value())
            .map(|(left, right)| left.binary(op, right))
            .transpose()?;
        let result = if matches!(op, B::LogicalAnd | B::LogicalOr) {
            let truth = match (op, self.truth(), right.truth()) {
                (B::LogicalAnd, Some(false), _) | (B::LogicalAnd, _, Some(false)) => Some(false),
                (B::LogicalOr, Some(true), _) | (B::LogicalOr, _, Some(true)) => Some(true),
                (_, Some(left), Some(right)) => Some(if op == B::LogicalAnd {
                    left && right
                } else {
                    left || right
                }),
                _ => None,
            };
            boolean(truth)?
        } else if matches!(op, B::ShiftLeft | B::ShiftRight) {
            shifts::binary(
                op,
                self.convert(result_type)?.range().integer()?,
                right.integer()?,
            )?
        } else {
            let common = self.ty().usual_arithmetic_conversion(right.ty());
            let (left, right) = (
                self.convert(common)?.range(),
                right.convert(common)?.range(),
            );
            match (left, right) {
                (Self::Integer(left), Self::Integer(right)) => {
                    integer_operations::binary(op, left, right)?
                }
                (Self::Float(_), Self::Float(_)) => {
                    if result_type == CScalarType::Int {
                        boolean(None)?
                    } else {
                        CTransfer::NonWrapping(Self::Float(FloatRange::unknown()))
                    }
                }
                _ => unreachable!("usual conversion selects one numeric category"),
            }
        };
        Ok(exact.map_or(result, |value| result.with_range(Self::exact(value))))
    }

    pub(in crate::ownership) fn unary(self, op: U) -> Result<CTransfer, E> {
        let ty = op.result_type(self.ty())?;
        let exact = self
            .exact_value()
            .map(|value| value.unary(op))
            .transpose()?;
        let result = if op == U::LogicalNot {
            boolean(self.truth().map(|value| !value))?
        } else {
            match self.convert(ty)?.range() {
                Self::Float(value) => {
                    let range = match value.bounds() {
                        Some((min, max)) => FloatRange::checked(-max, -min, value.may_nan())?,
                        None => FloatRange::exact(f64::NAN),
                    };
                    CTransfer::NonWrapping(Self::Float(range))
                }
                Self::Integer(value) => {
                    let (min, max) = if op == U::BitNot {
                        if matches!(ty.representation(), R::Unsigned(_)) {
                            let limit = limits(ty)?.1;
                            (limit - value.max(), limit - value.min())
                        } else {
                            (!value.max(), !value.min())
                        }
                    } else {
                        (-value.max(), -value.min())
                    };
                    integer_operations::result(value, min, max)?
                }
            }
        };
        Ok(exact.map_or(result, |value| result.with_range(Self::exact(value))))
    }
}
