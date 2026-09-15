//! Checked, target-independent built-in signed bitwise inputs.
use super::Capability;
use rustc_hir as hir;
use rustc_middle::ty::{self, TypeckResults};

pub(crate) struct IntegerBitwise;

#[derive(Clone, Copy)]
pub(crate) enum BitwiseOperator {
    And,
    Or,
    Xor,
}

#[derive(Clone, Copy)]
pub(crate) enum BitwiseOperands<'tcx> {
    Complement(&'tcx hir::Expr<'tcx>),
    Binary(
        BitwiseOperator,
        &'tcx hir::Expr<'tcx>,
        &'tcx hir::Expr<'tcx>,
    ),
}

pub(crate) struct BitwiseInput<'tcx> {
    operands: BitwiseOperands<'tcx>,
}

impl Capability for IntegerBitwise {
    type Input<'tcx> = BitwiseInput<'tcx>;
}

impl<'tcx> BitwiseInput<'tcx> {
    pub(crate) fn read(
        checked: &TypeckResults<'tcx>,
        expression: &'tcx hir::Expr<'tcx>,
    ) -> Result<Self, String> {
        let ty = checked.expr_ty(expression);
        if !matches!(ty.kind(), ty::Int(ty::IntTy::I32 | ty::IntTy::I64))
            || checked.type_dependent_def_id(expression.hir_id).is_some()
            || !checked.expr_adjustments(expression).is_empty()
        {
            return Err("integer bitwise requires an unadjusted built-in i32/i64 operation".into());
        }
        let check = |operand| {
            if checked.expr_ty(operand) != ty || !checked.expr_adjustments(operand).is_empty() {
                Err("integer bitwise operand type or adjustment differs".to_string())
            } else {
                Ok(())
            }
        };
        let operands = match expression.kind {
            hir::ExprKind::Unary(hir::UnOp::Not, operand) => {
                check(operand)?;
                BitwiseOperands::Complement(operand)
            }
            hir::ExprKind::Binary(operator, left, right) => {
                let operator = match operator.node {
                    hir::BinOpKind::BitAnd => BitwiseOperator::And,
                    hir::BinOpKind::BitOr => BitwiseOperator::Or,
                    hir::BinOpKind::BitXor => BitwiseOperator::Xor,
                    _ => return Err("unsupported integer bitwise operator".into()),
                };
                check(left)?;
                check(right)?;
                BitwiseOperands::Binary(operator, left, right)
            }
            _ => return Err("unsupported integer bitwise expression".into()),
        };
        Ok(Self { operands })
    }

    pub(crate) fn operands(&self) -> BitwiseOperands<'tcx> {
        self.operands
    }
}
