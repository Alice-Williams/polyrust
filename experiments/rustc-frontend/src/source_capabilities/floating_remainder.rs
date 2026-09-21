//! Canonical rustc evidence for built-in, unadjusted binary64 remainder.
use super::Capability;
use rustc_hir as hir;
use rustc_middle::ty::{self, TyCtxt, TypeckResults};

pub(crate) struct FloatingRemainder;
pub(crate) struct RemainderInput<'tcx> {
    source: &'tcx hir::Expr<'tcx>,
    left: &'tcx hir::Expr<'tcx>,
    right: &'tcx hir::Expr<'tcx>,
}
impl Capability for FloatingRemainder {
    type Input<'tcx> = RemainderInput<'tcx>;
}
impl<'tcx> RemainderInput<'tcx> {
    pub(crate) fn read(
        tcx: TyCtxt<'tcx>,
        checked: &TypeckResults<'tcx>,
        expression: &hir::Expr<'tcx>,
    ) -> Result<Self, String> {
        let hir::Node::Expr(canonical) = tcx.hir_node(expression.hir_id) else {
            return Err("floating remainder requires a canonical HIR expression".into());
        };
        if !std::ptr::eq(canonical, expression)
            || !std::ptr::eq(checked, tcx.typeck(expression.hir_id.owner.def_id))
        {
            return Err(
                "floating remainder requires its original checked function and HIR node".into(),
            );
        }
        let hir::ExprKind::Binary(operator, left, right) = canonical.kind else {
            return Err("floating remainder requires a binary expression".into());
        };
        if operator.node != hir::BinOpKind::Rem {
            return Err("floating remainder requires the built-in remainder operator".into());
        }
        let ty = checked.expr_ty(canonical);
        if checked.type_dependent_def_id(canonical.hir_id).is_some()
            || !matches!(ty.kind(), ty::Float(ty::FloatTy::F64))
            || checked.expr_ty(left) != ty
            || checked.expr_ty(right) != ty
            || !checked.expr_adjustments(canonical).is_empty()
            || !checked.expr_adjustments(left).is_empty()
            || !checked.expr_adjustments(right).is_empty()
        {
            return Err(
                "floating remainder requires unadjusted built-in f64 operands and result".into(),
            );
        }
        Ok(Self {
            source: canonical,
            left,
            right,
        })
    }
    pub(crate) fn require_context(
        &self,
        tcx: TyCtxt<'tcx>,
        checked: &TypeckResults<'tcx>,
    ) -> Result<(), String> {
        let current = Self::read(tcx, checked, self.source)?;
        if !std::ptr::eq(current.left, self.left) || !std::ptr::eq(current.right, self.right) {
            return Err("floating remainder operand identity differs".into());
        }
        Ok(())
    }
    pub(crate) fn left(&self) -> &'tcx hir::Expr<'tcx> {
        self.left
    }
    pub(crate) fn right(&self) -> &'tcx hir::Expr<'tcx> {
        self.right
    }
}

#[cfg(remainder_ast_probe)]
#[path = "../../test/remainder_input_probe.rs"]
mod probe;
