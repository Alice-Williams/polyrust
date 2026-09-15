//! Built-in eager Boolean operations, independent of target representations.
use super::Capability;
use rustc_hir as hir;
use rustc_middle::ty::{self, TypeckResults};

pub(crate) struct EagerBooleans;

#[derive(Clone, Copy)]
pub(crate) enum EagerBooleanOperator {
    And,
    Or,
    Xor,
}

pub(crate) struct EagerBooleanInput<'tcx> {
    operator: EagerBooleanOperator,
    left: &'tcx hir::Expr<'tcx>,
    right: &'tcx hir::Expr<'tcx>,
}

impl Capability for EagerBooleans {
    type Input<'tcx> = EagerBooleanInput<'tcx>;
}

impl<'tcx> EagerBooleanInput<'tcx> {
    pub(crate) fn read(
        checked: &TypeckResults<'tcx>,
        expression: &'tcx hir::Expr<'tcx>,
    ) -> Result<Self, String> {
        let hir::ExprKind::Binary(operator, left, right) = expression.kind else {
            return Err("eager Boolean input requires a binary expression".into());
        };
        if checked.type_dependent_def_id(expression.hir_id).is_some()
            || [expression, left, right].iter().any(|value| {
                !matches!(checked.expr_ty(value).kind(), ty::Bool)
                    || !checked.expr_adjustments(value).is_empty()
            })
        {
            return Err("eager Boolean input requires unadjusted built-in bool operands".into());
        }
        let operator = match operator.node {
            hir::BinOpKind::BitAnd => EagerBooleanOperator::And,
            hir::BinOpKind::BitOr => EagerBooleanOperator::Or,
            hir::BinOpKind::BitXor => EagerBooleanOperator::Xor,
            _ => return Err("unsupported eager Boolean operator".into()),
        };
        Ok(Self {
            operator,
            left,
            right,
        })
    }

    pub(crate) fn operator(&self) -> EagerBooleanOperator {
        self.operator
    }
    pub(crate) fn left(&self) -> &'tcx hir::Expr<'tcx> {
        self.left
    }
    pub(crate) fn right(&self) -> &'tcx hir::Expr<'tcx> {
        self.right
    }
}
