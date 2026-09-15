//! Lazy built-in bool operands; no target nodes or unchecked public fields.
use super::Capability;
use rustc_hir as hir;
use rustc_middle::ty::{self, TypeckResults};

pub(crate) struct ShortCircuitBooleans;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LazyBooleanOperator {
    And,
    Or,
}

pub(crate) struct LazyBooleanInput<'tcx> {
    operator: LazyBooleanOperator,
    left: &'tcx hir::Expr<'tcx>,
    right: &'tcx hir::Expr<'tcx>,
}

impl Capability for ShortCircuitBooleans {
    type Input<'tcx> = LazyBooleanInput<'tcx>;
}

impl<'tcx> LazyBooleanInput<'tcx> {
    pub(crate) fn read(
        checked: &TypeckResults<'tcx>,
        expression: &'tcx hir::Expr<'tcx>,
    ) -> Result<Self, String> {
        let hir::ExprKind::Binary(operator, left, right) = expression.kind else {
            return Err("short-circuit Boolean input requires a lazy binary operator".into());
        };
        let operator = match operator.node {
            hir::BinOpKind::And => LazyBooleanOperator::And,
            hir::BinOpKind::Or => LazyBooleanOperator::Or,
            _ => return Err("short-circuit Boolean input requires a lazy binary operator".into()),
        };
        if checked.type_dependent_def_id(expression.hir_id).is_some()
            || [expression, left, right].into_iter().any(|value| {
                !matches!(checked.expr_ty(value).kind(), ty::Bool)
                    || !checked.expr_adjustments(value).is_empty()
            })
        {
            return Err(
                "short-circuit Boolean input requires unadjusted built-in bool operands".into(),
            );
        }
        Ok(Self {
            operator,
            left,
            right,
        })
    }

    pub(crate) fn operator(&self) -> LazyBooleanOperator {
        self.operator
    }
    pub(crate) fn left(&self) -> &'tcx hir::Expr<'tcx> {
        self.left
    }
    pub(crate) fn right(&self) -> &'tcx hir::Expr<'tcx> {
        self.right
    }
}
