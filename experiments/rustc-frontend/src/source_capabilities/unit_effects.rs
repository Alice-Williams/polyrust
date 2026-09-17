//! Checked unit computations have effects, never a target value representation.
use super::Capability;
use rustc_hir as hir;
use rustc_middle::ty::{self, TypeckResults};

pub(crate) struct UnitEffects;

#[derive(Clone, Copy)]
pub(crate) enum UnitOperation<'tcx> {
    Empty,
    Call(&'tcx hir::Expr<'tcx>),
    Block(&'tcx hir::Expr<'tcx>),
    Conditional {
        condition: &'tcx hir::Expr<'tcx>,
        then_value: &'tcx hir::Expr<'tcx>,
        else_value: Option<&'tcx hir::Expr<'tcx>>,
    },
}

pub(crate) struct UnitInput<'tcx> {
    operation: UnitOperation<'tcx>,
    scope: hir::HirId,
}
impl Capability for UnitEffects {
    type Input<'tcx> = UnitInput<'tcx>;
}
impl<'tcx> UnitInput<'tcx> {
    pub(crate) fn read(
        checked: &TypeckResults<'tcx>,
        expression: &'tcx hir::Expr<'tcx>,
        scope: hir::HirId,
    ) -> Result<Self, String> {
        let unit = |value: &hir::Expr<'tcx>| {
            matches!(checked.expr_ty(value).kind(), ty::Tuple(fields) if fields.is_empty())
                && checked.expr_adjustments(value).is_empty()
                && checked.type_dependent_def_id(value.hir_id).is_none()
        };
        if scope.owner != expression.hir_id.owner || !unit(expression) {
            return Err("unit effects require an unadjusted built-in unit expression in the current function".into());
        }
        let operation = match expression.kind {
            hir::ExprKind::Tup([]) => UnitOperation::Empty,
            hir::ExprKind::Call(..) => UnitOperation::Call(expression),
            hir::ExprKind::Block(_, None) => UnitOperation::Block(expression),
            hir::ExprKind::If(condition, then_value, else_value)
                if matches!(checked.expr_ty(condition).kind(), ty::Bool)
                    && checked.expr_adjustments(condition).is_empty()
                    && unit(then_value)
                    && else_value.is_none_or(unit) =>
            {
                UnitOperation::Conditional {
                    condition,
                    then_value,
                    else_value,
                }
            }
            _ => return Err(
                "unsupported unit effect; early returns and arbitrary effects are not implemented"
                    .into(),
            ),
        };
        Ok(Self { operation, scope })
    }
    pub(crate) fn operation(&self) -> UnitOperation<'tcx> {
        self.operation
    }
    pub(crate) fn scope(&self) -> hir::HirId {
        self.scope
    }
}
