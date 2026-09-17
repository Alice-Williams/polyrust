//! Canonical checked built-in f64 negation, excluding overloaded Neg.
use super::Capability;
use rustc_hir as hir;
use rustc_middle::ty::{self, TyCtxt, TypeckResults};

pub(crate) struct FloatingNegation;
pub(crate) struct FloatingInput<'tcx> {
    source: &'tcx hir::Expr<'tcx>,
    operand: &'tcx hir::Expr<'tcx>,
}
impl Capability for FloatingNegation {
    type Input<'tcx> = FloatingInput<'tcx>;
}
impl<'tcx> FloatingInput<'tcx> {
    pub(crate) fn read(
        tcx: TyCtxt<'tcx>,
        checked: &TypeckResults<'tcx>,
        expression: &hir::Expr<'tcx>,
    ) -> Result<Self, String> {
        let hir::Node::Expr(canonical) = tcx.hir_node(expression.hir_id) else {
            return Err("floating negation requires a canonical HIR expression".into());
        };
        if !std::ptr::eq(canonical, expression)
            || !std::ptr::eq(checked, tcx.typeck(expression.hir_id.owner.def_id))
        {
            return Err(
                "floating negation requires its original checked function and HIR node".into(),
            );
        }
        let hir::ExprKind::Unary(hir::UnOp::Neg, operand) = canonical.kind else {
            return Err("floating negation requires unary minus".into());
        };
        if checked.type_dependent_def_id(canonical.hir_id).is_some()
            || !matches!(
                checked.expr_ty(canonical).kind(),
                ty::Float(ty::FloatTy::F64)
            )
            || checked.expr_ty(operand) != checked.expr_ty(canonical)
            || !checked.expr_adjustments(canonical).is_empty()
            || !checked.expr_adjustments(operand).is_empty()
        {
            return Err(
                "floating negation requires an unadjusted built-in f64 operand and result".into(),
            );
        }
        Ok(Self {
            source: canonical,
            operand,
        })
    }
    pub(crate) fn require_context(
        &self,
        tcx: TyCtxt<'tcx>,
        checked: &TypeckResults<'tcx>,
    ) -> Result<(), String> {
        let current = Self::read(tcx, checked, self.source)?;
        if !std::ptr::eq(current.operand, self.operand) {
            return Err("floating negation operand identity differs".into());
        }
        Ok(())
    }
    pub(crate) fn operand(&self) -> &'tcx hir::Expr<'tcx> {
        self.operand
    }
}

#[cfg(floating_ast_probe)]
#[path = "../../test/floating_input_probe.rs"]
mod input_probe;
