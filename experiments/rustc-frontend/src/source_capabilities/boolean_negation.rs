//! Built-in bool negation, independent of any target representation.
use super::Capability;
use rustc_hir as hir;
use rustc_middle::ty::{self, TypeckResults};

pub(crate) struct BooleanNegation;
pub(crate) struct NegationInput<'tcx> {
    operand: &'tcx hir::Expr<'tcx>,
}

impl Capability for BooleanNegation {
    type Input<'tcx> = NegationInput<'tcx>;
}

impl<'tcx> NegationInput<'tcx> {
    pub(crate) fn read(
        checked: &TypeckResults<'tcx>,
        expression: &'tcx hir::Expr<'tcx>,
    ) -> Result<Self, String> {
        let hir::ExprKind::Unary(hir::UnOp::Not, operand) = expression.kind else {
            return Err("Boolean negation requires unary Not".into());
        };
        if checked.type_dependent_def_id(expression.hir_id).is_some()
            || !matches!(checked.expr_ty(expression).kind(), ty::Bool)
            || !matches!(checked.expr_ty(operand).kind(), ty::Bool)
            || !checked.expr_adjustments(expression).is_empty()
            || !checked.expr_adjustments(operand).is_empty()
        {
            return Err("Boolean negation requires an unadjusted built-in bool operand".into());
        }
        Ok(Self { operand })
    }

    pub(crate) fn operand(&self) -> &'tcx hir::Expr<'tcx> {
        self.operand
    }
}
